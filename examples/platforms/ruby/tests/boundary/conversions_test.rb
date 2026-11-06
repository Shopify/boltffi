# frozen_string_literal: true

# Ruby-specific rules at the native boundary. These tests have no demo case:
# they pin the conversions every Ruby caller relies on.
class ConversionsTest < Minitest::Test
  def test_integers_reject_non_integers_instead_of_truncating
    assert_raises(TypeError) { Demo.echo_i32(1.5) }
    assert_raises(TypeError) { Demo.echo_i32("1") }
    assert_raises(TypeError) { Demo.echo_i32(nil) }
  end

  def test_conversion_methods_never_run
    integer_like = Object.new
    def integer_like.to_int = 1
    float_like = Object.new
    def float_like.to_f = 1.0
    assert_raises(TypeError) { Demo.echo_i32(integer_like) }
    assert_raises(TypeError) { Demo.echo_f64(float_like) }
  end

  def test_integers_reject_values_outside_the_rust_type
    assert_raises(RangeError) { Demo.echo_i8(128) }
    assert_raises(RangeError) { Demo.echo_i8(-129) }
    assert_raises(RangeError) { Demo.echo_i32(2**31) }
    assert_raises(RangeError) { Demo.echo_i64(2**63) }
    assert_raises(RangeError) { Demo.echo_isize(2**63) }
    assert_equal(-2**63, Demo.echo_i64(-2**63))
    assert_equal(2**62, Demo.echo_i64(2**62))
  end

  def test_unsigned_integers_reject_negative_and_oversized_values
    assert_raises(RangeError) { Demo.echo_u8(-1) }
    assert_raises(RangeError) { Demo.echo_u32(-1) }
    assert_raises(RangeError) { Demo.echo_u64(-1) }
    assert_raises(RangeError) { Demo.echo_usize(-1) }
    # `NUM2ULL` wraps a negative Bignum that fits in 64 bits, so only the sign
    # check rejects this one.
    assert_raises(RangeError) { Demo.echo_u64(-2**63) }
    assert_raises(RangeError) { Demo.echo_u64(-2**64) }
    assert_raises(RangeError) { Demo.echo_u64(2**64) }
    assert_raises(RangeError) { Demo.echo_usize(2**64) }
    assert_raises(RangeError) { Demo.echo_u16(65_536) }
    assert_raises(RangeError) { Demo.echo_u32(2**32) }
    assert_equal(2**63, Demo.echo_u64(2**63))
    assert_equal(2**64 - 1, Demo.echo_u64(2**64 - 1))
  end

  def test_floats_accept_integers_and_reject_other_types
    assert_equal(3.0, Demo.echo_f64(3))
    assert_equal(2.0**70, Demo.echo_f64(2**70))
    assert_raises(TypeError) { Demo.echo_f64("3.0") }
    assert_raises(TypeError) { Demo.echo_f64(nil) }
    assert_raises(RangeError) { Demo.echo_f32(1e39) }
    assert_equal(Float::INFINITY, Demo.echo_f32(Float::INFINITY))
    assert_equal(2.0, Demo.echo_f32(2))
    assert_predicate(Demo.echo_f64(Float::NAN), :nan?)
  end

  # Ruby warns when an Integer is too large for a Float. A warning hook is Ruby
  # code, and no Ruby code may run while an argument is converted.
  def test_integers_too_large_for_a_float_raise_without_a_warning
    verbose = $VERBOSE
    $VERBOSE = true
    assert_silent do
      assert_raises(RangeError) { Demo.echo_f64(2**1100) }
    end
  ensure
    $VERBOSE = verbose
  end

  def test_booleans_are_strict
    assert_raises(TypeError) { Demo.echo_bool(nil) }
    assert_raises(TypeError) { Demo.echo_bool(1) }
    assert_equal(false, Demo.echo_bool(false))
  end

  # The extension is not Ractor-safe, so Ruby refuses calls from other Ractors.
  def test_calls_from_other_ractors_raise
    experimental = Warning[:experimental]
    Warning[:experimental] = false
    ractor = Ractor.new do
      Thread.current.report_on_exception = false
      Demo.noop
    end
    error = assert_raises(Ractor::RemoteError) do
      ractor.respond_to?(:value) ? ractor.value : ractor.take
    end
    assert_kind_of(Ractor::UnsafeError, error.cause)
  ensure
    Warning[:experimental] = experimental
  end

  # String tests below are ported from Max VelDink's ruby-lean-target draft.
  def test_strings_must_be_valid_utf8
    error = assert_raises(ArgumentError) { Demo.echo_string("\xff") }
    assert_match(/invalid byte sequence in UTF-8/, error.message)
    assert_raises(Encoding::CompatibilityError) { Demo.echo_string("caf\u00e9".b) }
    assert_raises(Encoding::CompatibilityError) { Demo.echo_string("caf\u00e9".encode("ISO-8859-1")) }
    assert_raises(TypeError) { Demo.echo_string(:symbol) }
    assert_raises(TypeError) { Demo.echo_string(nil) }
  end

  def test_strings_are_strict_and_never_call_to_str
    convertible = Object.new
    def convertible.to_str = raise("to_str must not run")
    subclass = Class.new(String)

    assert_raises(TypeError) { Demo.echo_string(convertible) }
    assert_raises(TypeError) { Demo.echo_bytes(convertible) }
    assert_equal("text", Demo.echo_string(subclass.new("text")))
  end

  def test_ascii_only_strings_pass_in_any_ascii_compatible_encoding
    assert_equal("abc", Demo.echo_string("abc".b))
    assert_equal("abc", Demo.echo_string("abc".encode("US-ASCII")))
  end

  def test_returned_strings_are_utf8
    result = Demo.echo_string("caf\u00e9")

    assert_equal(Encoding::UTF_8, result.encoding)
    assert_predicate(result, :valid_encoding?)
  end

  # The extension sets the coderange itself. Covers the 32-byte chunks and the tail.
  def test_returned_strings_report_ascii_only_correctly
    assert_predicate(Demo.echo_string("plain"), :ascii_only?)
    assert_predicate(Demo.echo_string("x" * 64), :ascii_only?)
    refute_predicate(Demo.echo_string("x" * 40 + "\u00e9"), :ascii_only?)
    refute_predicate(Demo.echo_string("\u00e9" + "x" * 40), :ascii_only?)
    assert_predicate(Demo.echo_string("x" * 40 + "\u00e9"), :valid_encoding?)
  end

  # Bytes are raw: any encoding goes in, a binary String comes out.
  def test_bytes_accept_any_string_and_return_binary
    assert_equal("\xff\x00".b, Demo.echo_bytes("\xff\x00".b))
    assert_equal("caf\u00e9".b, Demo.echo_bytes("caf\u00e9"))
    assert_equal(Encoding::BINARY, Demo.echo_bytes("abc").encoding)
    assert_raises(TypeError) { Demo.echo_bytes([1, 2, 3]) }
  end

  # Values past the 128-byte inline buffer move into a GC-owned String.
  def test_large_values_cross_in_both_directions
    text = "\u00e9" * 100_000
    bytes = Random.new(1).bytes(100_000)

    assert_equal(text, Demo.echo_string(text))
    assert_equal(bytes, Demo.echo_bytes(bytes))
    assert_equal(text * 3, Demo.repeat_string(text, 3))
  end

  def test_strings_survive_garbage_collection_on_every_allocation
    GC.stress = true
    assert_equal("b" * 300, Demo.echo_string("b" * 300))
    assert_equal("ab", Demo.concat_strings("a", "b"))
  ensure
    GC.stress = false
  end

  # A spilled buffer can sit inside its String object, and compaction moves
  # objects. The writer keeps it pinned until Rust has read it.
  def test_spilled_arguments_survive_compaction
    auto_compact = GC.auto_compact
    GC.auto_compact = true
    GC.stress = 0x01 | 0x04
    text = "\u00e9" * 200
    20.times { assert_equal(text, Demo.echo_string(text)) }
  ensure
    GC.stress = false
    GC.auto_compact = auto_compact
  end

  def test_a_bad_argument_after_a_string_leaves_the_process_healthy
    1_000.times do
      assert_raises(RangeError) { Demo.repeat_string("x" * 1_000, -1) }
    end
    GC.start

    assert_equal("okok", Demo.repeat_string("ok", 2))
  end
end
