use crate::{ResultComp, ResultCompErrExt};

pub trait MqiOption<P> {
    fn apply_param(self, param: &mut P);
}

impl<P, T: MqiOption<P>> MqiOption<P> for Option<T> {
    fn apply_param(self, param: &mut P) {
        if let Some(option) = self {
            option.apply_param(param);
        }
    }
}

impl<F, P> MqiOption<P> for F
where
    F: FnOnce(&mut P),
{
    fn apply_param(self, param: &mut P) {
        self(param);
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
    type Param<'p> = T::Param<'p>;

    fn from_mqi<F>(param: &mut Self::Param<'_>, mqi: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut Self::Param<'_>) -> ResultComp<T>,
    {
        let (a, mqi_result) = A::from_mqi(param, |p| T::from_mqi(p, mqi));
        mqi_result.map_completion(move |mqi| (mqi, a))
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
        let ((a, b), mqi_result) = <(A, B) as MqiAttr<Self::Param<'_>>>::from_mqi(param, |p| T::from_mqi(p, mqi));
        mqi_result.map_completion(|mqi| (mqi, a, b))
    }
}

impl<T, A, B, C> MqiValue<T> for (T, A, B, C)
where
    T: MqiValue<T>,
    A: for<'a> MqiAttr<T::Param<'a>>,
    B: for<'a> MqiAttr<T::Param<'a>>,
    C: for<'a> MqiAttr<T::Param<'a>>,
{
    type Param<'a> = T::Param<'a>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<T>>(param: &mut Self::Param<'_>, mqi: F) -> ResultComp<Self> {
        let ((a, b, c), mqi_result) = <(A, B, C) as MqiAttr<Self::Param<'_>>>::from_mqi(param, |p| T::from_mqi(p, mqi));
        mqi_result.map_completion(|mqi| (mqi, a, b, c))
    }
}

impl<T, A, B, C, D> MqiValue<T> for (T, A, B, C, D)
where
    T: MqiValue<T>,
    A: for<'a> MqiAttr<T::Param<'a>>,
    B: for<'a> MqiAttr<T::Param<'a>>,
    C: for<'a> MqiAttr<T::Param<'a>>,
    D: for<'a> MqiAttr<T::Param<'a>>,
{
    type Param<'a> = T::Param<'a>;

    fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<T>>(param: &mut Self::Param<'_>, mqi: F) -> ResultComp<Self> {
        let ((a, b, c, d), mqi_result) = <(A, B, C, D) as MqiAttr<Self::Param<'_>>>::from_mqi(param, |p| T::from_mqi(p, mqi));
        mqi_result.map_completion(|mqi| (mqi, a, b, c, d))
    }
}

pub trait MqiAttr<P>: Sized {
    fn from_mqi<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y);
}

impl<P> MqiAttr<P> for () {
    #[inline]
    fn from_mqi<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y) {
        ((), mqi(param))
    }
}

impl<A, B, P> MqiAttr<P> for (A, B)
where
    A: MqiAttr<P>,
    B: MqiAttr<P>,
{
    #[inline]
    fn from_mqi<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y) {
        let (a, (b, y)) = A::from_mqi(param, |f| B::from_mqi(f, mqi));
        ((a, b), y)
    }
}

impl<A, B, C, P> MqiAttr<P> for (A, B, C)
where
    A: MqiAttr<P>,
    B: MqiAttr<P>,
    C: MqiAttr<P>,
{
    #[inline]
    fn from_mqi<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y) {
        let (a, ((b, c), y)) = A::from_mqi(param, |f| <(B, C) as MqiAttr<P>>::from_mqi(f, mqi));
        ((a, b, c), y)
    }
}

#[allow(clippy::many_single_char_names)]
impl<A, B, C, D, P> MqiAttr<P> for (A, B, C, D)
where
    A: MqiAttr<P>,
    B: MqiAttr<P>,
    C: MqiAttr<P>,
    D: MqiAttr<P>,
{
    #[inline]
    fn from_mqi<Y, F: FnOnce(&mut P) -> Y>(param: &mut P, mqi: F) -> (Self, Y) {
        let (a, ((b, c, d), y)) = A::from_mqi(param, |f| <(B, C, D) as MqiAttr<P>>::from_mqi(f, mqi));
        ((a, b, c, d), y)
    }
}

impl<P> MqiOption<P> for () {
    fn apply_param(self, _param: &mut P) {}
}

impl<P, A, B> MqiOption<P> for (A, B)
where
    A: MqiOption<P>,
    B: MqiOption<P>,
{
    fn apply_param(self, param: &mut P) {
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<P, A, B, C> MqiOption<P> for (A, B, C)
where
    A: MqiOption<P>,
    B: MqiOption<P>,
    C: MqiOption<P>,
{
    fn apply_param(self, param: &mut P) {
        self.2.apply_param(param);
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<P, A, B, C, D> MqiOption<P> for (A, B, C, D)
where
    A: MqiOption<P>,
    B: MqiOption<P>,
    C: MqiOption<P>,
    D: MqiOption<P>,
{
    fn apply_param(self, param: &mut P) {
        self.3.apply_param(param);
        self.2.apply_param(param);
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<P, A, B, C, D, E> MqiOption<P> for (A, B, C, D, E)
where
    A: MqiOption<P>,
    B: MqiOption<P>,
    C: MqiOption<P>,
    D: MqiOption<P>,
    E: MqiOption<P>,
{
    fn apply_param(self, param: &mut P) {
        self.4.apply_param(param);
        self.3.apply_param(param);
        self.2.apply_param(param);
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}

impl<P, A, B, C, D, E, F> MqiOption<P> for (A, B, C, D, E, F)
where
    A: MqiOption<P>,
    B: MqiOption<P>,
    C: MqiOption<P>,
    D: MqiOption<P>,
    E: MqiOption<P>,
    F: MqiOption<P>,
{
    fn apply_param(self, param: &mut P) {
        self.5.apply_param(param);
        self.4.apply_param(param);
        self.3.apply_param(param);
        self.2.apply_param(param);
        self.1.apply_param(param);
        self.0.apply_param(param);
    }
}
