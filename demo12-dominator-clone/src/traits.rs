#![allow(unused)]
#![allow(unused_imports)]
use crate::{EventOptions, RefFn};
use std::borrow::Cow;

pub trait StaticEvent {
    const EVENT_TYPE: &'static str;
    fn unchecked_from_event(event: web_sys::Event) -> Self;

    #[inline] // 内联会将代码编译时去掉函数封装。https://nihil.cc/posts/translate_rust_inline/
    fn default_options(preventable: bool) -> EventOptions {
        if preventable {
            EventOptions::preventable()
        } else {
            EventOptions::default()
        }
    }
}

pub trait AsStr {
    #[deprecated(note = "Use with_str instead.")]
    fn as_str(&self) -> &str;

    fn with_str<A, F>(&self, f: F) -> A
    where
        F: FnOnce(&str) -> A, {
        #[allow(deprecated)]
        f(self.as_str())
    }
}

impl<'a, A> AsStr for &'a A where A: AsStr {
    fn as_str(&self) -> &str 
    {
        #[allow(deprecated)]
        AsStr::as_str(*self)
    }

    fn with_str<B, F>(&self, f: F) -> B
        where
            F: FnOnce(&str) -> B
    {
        AsStr::with_str(*self, f)
    }
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }

    fn with_str<A, F>(&self, f: F) -> A
        where
            F: FnOnce(&str) -> A
    {
        f(&self)   
    }
}

impl AsStr for str {
    fn as_str(&self) -> &str {
        self
    }

    fn with_str<A, F>(&self, f: F) -> A
        where
            F: FnOnce(&str) -> A, {
        f(self)
    }
}

impl<'a> AsStr for &'a str {
    fn as_str(&self) -> &str {
        self
    }
    fn with_str<A, F>(&self, f: F) -> A
        where
            F: FnOnce(&str) -> A, {
        f(self)
    }
}

impl<'a> AsStr for Cow<'a, str> {
    fn as_str(&self) -> &str {
        &*self
    }
    fn with_str<A, F>(&self, f: F) -> A
        where
            F: FnOnce(&str) -> A, {
        f(&*self)
    }
}

// TODO: 这里直接调用函数, 与函数名表达含义不匹配
impl<A, C> AsStr for RefFn<A, str, C> where C: Fn(&A) -> &str {
    fn as_str(&self) -> &str {
        self.call_ref()
    }
    fn with_str<B, F>(&self, f: F) -> B
        where
            F: FnOnce(&str) -> B, {
        f(self.call_ref())
    }
}


#[derive(Debug)]
pub(crate) struct MapMultiStr<A, F> where F: Fn(&str) -> String {
    multi_str: A,
    callback: F,
}

impl<A, F> MapMultiStr<A, F> where F: Fn(&str) -> String {
    pub(crate) fn new(multi_str: A, callback: F) -> Self {
        Self { multi_str, callback }
    }
}

pub trait MultiStr {
    fn find_map<A, F>(&self, f: F) -> Option<A> where F: FnMut(&str) -> Option<A>;

    fn each<F>(&self, mut f: F) where F: FnMut(&str) {
        let _: Option<()> = self.find_map(|x| {
            f(x);
            None
        });
    }
}

impl<A> MultiStr for A where A: AsStr {
    fn find_map<B, F>(&self, f: F) -> Option<B> where F: FnMut(&str) -> Option<B> {
        self.with_str(f)
    }
}

impl<M, T> MultiStr for MapMultiStr<M, T> where M: MultiStr, T: Fn(&str) -> String {
    fn find_map<A, F>(&self, mut f: F) -> Option<A> where F: FnMut(&str) -> Option<A> {
        self.multi_str.find_map(|x| {
            f(&(self.callback)(x))
        })
    }
}

impl<'a, A, C> MultiStr for RefFn<A, [&'a str], C> where C: Fn(&A) -> &[&'a str] {
    fn find_map<B, F>(&self, mut f: F) -> Option<B> where F: FnMut(&str) -> Option<B> {
        self.call_ref().iter().find_map(|x| f(x))
    }
}

macro_rules! array_multi_str {
    ($size:expr) => {
        impl<A> MultiStr for [A; $size] where A: AsStr {
            #[inline]
            fn find_map<B, F>(&self, mut f: F) -> Option<B> where F: FnMut(&str) -> Option<B> {
                self.iter().find_map(|x| x.with_str(|x| f(x)))
            }
        }
    }
}

macro_rules! array_multi_strs {
    ($($size:expr),*) => {
        $(array_multi_str!($size);)*
    };
}

array_multi_strs!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32);

pub trait OptionStr {
    type Output;

    fn into_option(self) -> Option<Self::Output>;
}

impl<A> OptionStr for A where A: MultiStr {
    type Output = A;

    fn into_option(self) -> Option<A> {
        Some(self)
    }
}

impl<A> OptionStr for Option<A> where A: MultiStr {
    type Output = A;

    fn into_option(self) -> Option<Self::Output> {
        self
    }
}