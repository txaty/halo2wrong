use super::AssignedPoint;
use super::BaseFieldEccChip;
use crate::halo2;
use crate::halo2::plonk::ErrorFront;
use halo2::arithmetic::CurveAffine;
use integer::maingate::RegionCtx;
use integer::IntegerInstructions;

impl<C: CurveAffine, const NUMBER_OF_LIMBS: usize, const BIT_LEN_LIMB: usize>
    BaseFieldEccChip<C, NUMBER_OF_LIMBS, BIT_LEN_LIMB>
{
    /// Optimized point addition algorithm
    ///
    /// The operands `a` and `b` must be distinct and neither
    /// of them can be the point at infinity.
    /// Otherwise the function returns an erroneous point.
    pub(crate) fn _add_incomplete_unsafe(
        &self,
        ctx: &mut RegionCtx<'_, C::Scalar>,
        a: &AssignedPoint<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
        b: &AssignedPoint<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
    ) -> Result<AssignedPoint<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>, ErrorFront> {
        let ch = self.integer_chip();

        // lambda = b_y - a_y / b_x - a_x
        let numerator = &ch.sub(ctx, &b.y, &a.y)?;
        let denominator = &ch.sub(ctx, &b.x, &a.x)?;
        let lambda = &ch.div_incomplete(ctx, numerator, denominator)?;

        // c_x =  lambda * lambda - a_x - b_x
        let lambda_square = &ch.square(ctx, lambda)?;
        let x = &ch.sub_sub(ctx, lambda_square, &a.x, &b.x)?;

        // c_y = lambda * (a_x - c_x) - a_y
        let t = &ch.sub(ctx, &a.x, x)?;
        let t = &ch.mul(ctx, t, lambda)?;
        let y = ch.sub(ctx, t, &a.y)?;

        let p_0 = AssignedPoint::new(x.clone(), y);

        Ok(p_0)
    }

    /// Optimized point doubling algorithm
    ///
    /// The point provided must not be the point at infinity
    pub(crate) fn _double_incomplete(
        &self,
        ctx: &mut RegionCtx<'_, C::Scalar>,
        point: &AssignedPoint<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
    ) -> Result<AssignedPoint<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>, ErrorFront> {
        let integer_chip = self.integer_chip();

        // lambda = (3 * a_x^2) / 2 * a_y
        let x_0_square = &integer_chip.square(ctx, &point.x)?;
        let numerator = &integer_chip.mul3(ctx, x_0_square)?;
        let denominator = &integer_chip.mul2(ctx, &point.y)?;
        let lambda = &integer_chip.div_incomplete(ctx, numerator, denominator)?;

        // c_x = lambda * lambda - 2 * a_x
        let lambda_square = &integer_chip.square(ctx, lambda)?;
        let x = &integer_chip.sub_sub(ctx, lambda_square, &point.x, &point.x)?;

        // c_y = lambda * (a_x - c_x) - a_y
        let t = &integer_chip.sub(ctx, &point.x, x)?;
        let t = &integer_chip.mul(ctx, lambda, t)?;
        let y = integer_chip.sub(ctx, t, &point.y)?;

        Ok(AssignedPoint::new(x.clone(), y))
    }

    /// Given 2 `AssignedPoint` $P$ and $Q$ efficiently computes $2*P + Q$
    ///
    /// see: https://hackmd.io/ncuKqRXzR-Cw-Au2fGzsMg?view
    pub(crate) fn _ladder_incomplete(
        &self,
        ctx: &mut RegionCtx<'_, C::Scalar>,
        to_double: &AssignedPoint<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
        to_add: &AssignedPoint<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>,
    ) -> Result<AssignedPoint<C::Base, C::Scalar, NUMBER_OF_LIMBS, BIT_LEN_LIMB>, ErrorFront> {
        let ch = self.integer_chip();

        // (P + Q) + P
        // P is to_double (x_1, y_1)
        // Q is to_add (x_2, y_2)

        // lambda_0 = (y_2 - y_1) / (x_2 - x_1)
        let numerator = &ch.sub(ctx, &to_add.y, &to_double.y)?;
        let denominator = &ch.sub(ctx, &to_add.x, &to_double.x)?;
        let lambda_0 = &ch.div_incomplete(ctx, numerator, denominator)?;

        // x_3 = lambda_0 * lambda_0 - x_1 - x_2
        let lambda_0_square = &ch.square(ctx, lambda_0)?;
        let x_3 = &ch.sub_sub(ctx, lambda_0_square, &to_add.x, &to_double.x)?;

        // lambda_1 = lambda_0 + 2 * y_1 / (x_3 - x_1)
        let numerator = &ch.mul2(ctx, &to_double.y)?;
        let denominator = &ch.sub(ctx, x_3, &to_double.x)?;
        let lambda_1 = &ch.div_incomplete(ctx, numerator, denominator)?;
        let lambda_1 = &ch.add(ctx, lambda_0, lambda_1)?;

        // x_4 = lambda_1 * lambda_1 - x_1 - x_3
        let lambda_1_square = &ch.square(ctx, lambda_1)?;
        let x_4 = &ch.sub_sub(ctx, lambda_1_square, x_3, &to_double.x)?;

        // y_4 = lambda_1 * (x_4 - x_1) - y_1
        let t = &ch.sub(ctx, x_4, &to_double.x)?;
        let t = &ch.mul(ctx, t, lambda_1)?;
        let y_4 = ch.sub(ctx, t, &to_double.y)?;

        let p_0 = AssignedPoint::new(x_4.clone(), y_4);

        Ok(p_0)
    }
}
