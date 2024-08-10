use crate::{ResultComp, ResultCompErrExt};

pub trait MqiOption<'a, P: 'a> {
    fn apply_param(&self, param: &mut P);
}

impl<'a, P: 'a, T: MqiOption<'a, P>> MqiOption<'a, P> for Option<T> {
    fn apply_param(&self, param: &mut P) {
        if let Some(option) = self {
            option.apply_param(param);
        }
    }
}

pub trait MqiValue<T>: Sized {
    type Param<'a>;

    #[allow(unused_variables)]
    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<T>>(param: &mut Self::Param<'_>, mqi: F) -> ResultComp<Self>;
}

impl<T, A> MqiValue<T> for (T, A)
where
    T: MqiValue<T>,
    A: for<'a> MqiAttr<T::Param<'a>>,
{
    type Param<'a> = T::Param<'a>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<T>>(param: &mut Self::Param<'_>, mqi: F) -> ResultComp<Self> {
        let (a, mqi_result) = A::apply_param(param, |p| T::from_mqi(p, mqi));
        mqi_result.map_completion(|mqi| (mqi, a))
    }
}

impl<T, A, B> MqiValue<T> for (T, A, B)
where
    T: MqiValue<T>,
    A: for<'a> MqiAttr<T::Param<'a>>,
    B: for<'a> MqiAttr<T::Param<'a>>,
{
    type Param<'a> = T::Param<'a>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<T>>(param: &mut Self::Param<'_>, mqi: F) -> ResultComp<Self> {
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

impl<'a, P: 'a> MqiOption<'a, P> for () {
    fn apply_param(&self, _param: &mut P) {}
}

impl<'a, P, A, B> MqiOption<'a, P> for (A, B)
where
    P: 'a,
    A: MqiOption<'a, P>,
    B: MqiOption<'a, P>,
{
    fn apply_param(&self, param: &mut P) {
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<'a, P, A, B, C> MqiOption<'a, P> for (A, B, C)
where
    P: 'a,
    A: MqiOption<'a, P>,
    B: MqiOption<'a, P>,
    C: MqiOption<'a, P>,
{
    fn apply_param(&self, param: &mut P) {
        self.2.apply_param(param);
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<'a, P, A, B, C, D> MqiOption<'a, P> for (A, B, C, D)
where
    P: 'a,
    A: MqiOption<'a, P>,
    B: MqiOption<'a, P>,
    C: MqiOption<'a, P>,
    D: MqiOption<'a, P>,
{
    fn apply_param(&self, param: &mut P) {
        self.3.apply_param(param);
        self.2.apply_param(param);
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<'a, F, P> MqiOption<'a, P> for F
where
    F: FnOnce(&mut P) + Copy,
    P: 'a,
{
    #[inline]
    fn apply_param(&self, param: &mut P) {
        self(param);
    }
}
