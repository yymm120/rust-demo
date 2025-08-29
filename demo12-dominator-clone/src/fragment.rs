use web_sys::Node;

use crate::callbacks::Callbacks;



pub trait Fragment {
    fn apply<'a>(&self, dom: FragmentBuilder<'a>) -> FragmentBuilder<'a> {
        (*self).apply(dom)
    }
}

#[derive(Debug)]
pub struct DomBuilder<A> {
    element: A,
    callbacks: Callbacks,
}

#[derive(Debug)]
pub struct FragmentBuilder<'a>(pub(crate) DomBuilder<&'a Node>);

pub type BoxFragment = Box<dyn Fragment + Send + Sync>;
