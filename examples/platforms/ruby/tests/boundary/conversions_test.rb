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
end
