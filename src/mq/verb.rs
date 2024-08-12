use crate::{ResultComp, ResultCompErrExt};

pub trait MqiOption<'p, P> {
    fn apply_param(&'p self, param: &mut P);
}

impl<'p, P, T: MqiOption<'p, P>> MqiOption<'p, P> for Option<T> {
    fn apply_param(&'p self, param: &mut P) {
        if let Some(option) = self {
            option.apply_param(param);
        }
    }
}

pub trait MqiValue<'b, T>: Sized {
    type Param<'a>;

    #[allow(unused_variables)]
    fn from_mqi<F: FnOnce(&mut Self::Param<'b>) -> ResultComp<T>>(param: &mut Self::Param<'b>, mqi: F) -> ResultComp<Self>;
}

impl<'b, T, A> MqiValue<'b, T> for (T, A)
where
    T: MqiValue<'b, T>,
    A: MqiAttr<T::Param<'b>>,
{
    type Param<'a> = T::Param<'a>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'b>) -> ResultComp<T>>(param: &mut Self::Param<'b>, mqi: F) -> ResultComp<Self> {
        let (a, mqi_result) = A::apply_param(param, |p| T::from_mqi(p, mqi));
        mqi_result.map_completion(move |mqi| (mqi, a))
    }
}

impl<'b, T, A, B> MqiValue<'b, T> for (T, A, B)
where
    T: MqiValue<'b, T>,
    A: MqiAttr<T::Param<'b>>,
    B: MqiAttr<T::Param<'b>>,
{
    type Param<'a> = T::Param<'a>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'b>) -> ResultComp<T>>(param: &mut Self::Param<'b>, mqi: F) -> ResultComp<Self> {
        let ((a, b), mqi_result) = <(A, B) as MqiAttr<Self::Param<'_>>>::apply_param(param, |p| T::from_mqi(p, mqi));
        mqi_result.map_completion(|mqi| (mqi, a, b))
    }
}

pub trait MqiAttr<P>: Sized {
    fn apply_param<Y, F: for<'a> FnOnce(&'a mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y);
}

impl<P> MqiAttr<P> for () {
    fn apply_param<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y) {
        ((), mqi(param))
    }
}

impl<A, B, P> MqiAttr<P> for (A, B)
where
    A: MqiAttr<P>,
    B: MqiAttr<P>,
{
    fn apply_param<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y) {
        let (a, (b, y)) = A::apply_param(param, |f| B::apply_param(f, mqi));
        ((a, b), y)
    }
}

impl<'p, P> MqiOption<'p, P> for () {
    fn apply_param(&self, _param: &mut P) {}
}

impl<'p, P, A, B> MqiOption<'p, P> for (A, B)
where
    A: MqiOption<'p, P>,
    B: MqiOption<'p, P>,
{
    fn apply_param(&'p self, param: &mut P) {
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<'p, P, A, B, C> MqiOption<'p, P> for (A, B, C)
where
    A: MqiOption<'p, P>,
    B: MqiOption<'p, P>,
    C: MqiOption<'p, P>,
{
    fn apply_param(&'p self, param: &mut P) {
        self.2.apply_param(param);
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<'p, P, A, B, C, D> MqiOption<'p, P> for (A, B, C, D)
where
    A: MqiOption<'p, P>,
    B: MqiOption<'p, P>,
    C: MqiOption<'p, P>,
    D: MqiOption<'p, P>,
{
    fn apply_param(&'p self, param: &mut P) {
        self.3.apply_param(param);
        self.2.apply_param(param);
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<F, P> MqiOption<'_, P> for F
where
    F: FnOnce(&mut P) + Copy,
{
    #[inline]
    fn apply_param(&self, param: &mut P) {
        self(param);
    }
}
