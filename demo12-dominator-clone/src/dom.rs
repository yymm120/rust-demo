use futures_signals::signal::Signal;
use wasm_bindgen::{intern, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{
    CssRule,  CssStyleDeclaration, CssStyleRule, CssStyleSheet, 
};

use crate::{
    bindings,
    callbacks::Callbacks,
    operations::for_each,
    traits::{AsStr, MultiStr, OptionStr},
};
use crate::utils::{UnwrapJsExt};

pub struct RefFn<A, B, C>
where
    B: ?Sized,
    C: Fn(&A) -> &B,
{
    value: A,
    callback: C,
}

impl<A, B, C> RefFn<A, B, C>
where
    B: ?Sized,
    C: Fn(&A) -> &B,
{
    pub fn new(value: A, callback: C) -> Self {
        Self { value, callback }
    }

    pub fn call_ref(&self) -> &B {
        (self.callback)(&self.value)
    }
}

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

pub const HIGHEST_ZINDEX: &str = "2147483647";

// static HIDDEN_CLASS: Lazy<String> = Lazy::new(|| class! {
//     .style_important("display", "none")
// });

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EventOptions {
    pub bubbles: bool,
    pub preventable: bool,
}

impl EventOptions {
    pub fn bubbles() -> Self {
        Self {
            bubbles: true,
            preventable: false,
        }
    }

    pub fn preventable() -> Self {
        Self {
            bubbles: false,
            preventable: true,
        }
    }
}

impl Default for EventOptions {
    fn default() -> Self {
        Self {
            bubbles: false,
            preventable: false,
        }
    }
}

pub mod __internal {
    use crate::fragment::{BoxFragment, Fragment, FragmentBuilder};
    use crate::traits::MultiStr;
    use std::sync::atomic::{AtomicU32, Ordering};

    pub use web_sys::HtmlElement;
    pub use web_sys::SvgElement;

    pub fn make_class_id(name: Option<&str>) -> String {
        static CLASS_ID: AtomicU32 = AtomicU32::new(0);

        let id = CLASS_ID.fetch_add(1, Ordering::Relaxed);
        let name = name.unwrap_or("__class_");
        format!("{}_{}", name, id)
    }
}

#[must_use]
pub struct ClassBuilder {
    stylesheet: StylesheetBuilder,
    class_name: String,
}

impl ClassBuilder {
    pub fn __internal_new(name: Option<&str>) -> Self {
        let class_name = __internal::make_class_id(name);
        Self {
            stylesheet: StylesheetBuilder::__internal_stylesheet(&format!(".{} {{}}", class_name)),
            class_name,
        }
    }

    pub fn __internal_class_name(&self) -> &str {
        &self.class_name
    }

    pub fn style<B, C>(mut self, name: B, value: C) -> Self
    where
        B: MultiStr,
        C: MultiStr,
    {
        self.stylesheet = self.stylesheet.style(name, value);
        self
    }

    pub fn style_important<B, C>(mut self, name: B, value: C) -> Self
    where
        B: MultiStr,
        C: MultiStr,
    {
        self.stylesheet = self.stylesheet.style_important(name, value);
        self
    }

    pub fn style_unchecked<B, C>(mut self, name: B, value: C) -> Self
    where
        B: AsStr,
        C: AsStr,
    {
        self.stylesheet = self.stylesheet.style_unchecked(name, value);
        self
    }

    pub fn style_signal<B, C, D, E>(mut self, name: B, value: E) -> Self
    where
        B: MultiStr + 'static,
        C: MultiStr,
        D: OptionStr<Output = C>,
        E: Signal<Item = D> + 'static,
    {
        self.stylesheet = self.stylesheet.style_signal(name, value);
        self
    }

    pub fn style_import_signal<B, C, D, E>(mut self, name: B, value: E) -> Self
    where
        B: MultiStr + 'static,
        C: MultiStr,
        D: OptionStr<Output = C>,
        E: Signal<Item = D> + 'static,
    {
        self.stylesheet = self.stylesheet.style_important_signal(name, value);
        self
    }

    pub fn style_unchecked_signal<B, C, D, E>(mut self, name: B, value: E) -> Self
    where
        B: AsStr + 'static,
        C: AsStr,
        D: OptionStr<Output = C>,
        E: Signal<Item = D> + 'static,
    {
        self.stylesheet = self.stylesheet.style_unchecked_signal(name, value);
        self
    }

    pub fn raw<B>(mut self, css: B) -> Self
    where
        B: AsStr,
    {
        self.stylesheet = self.stylesheet.raw(css);
        self
    }

    pub fn __internal_done(self) -> String {
        self.stylesheet.__internal_done();
        self.class_name
    }
}

#[must_use]
pub struct StylesheetBuilder {
    element: CssStyleDeclaration,
    callbacks: Callbacks,
}

impl StylesheetBuilder {
    fn __internal_rules<A>(rules: &A) -> CssRule
    where
        A: MultiStr,
    {
        thread_local! {
            static STYLESHEET: CssStyleSheet = bindings::create_stylesheet(None);
        }
        STYLESHEET.with(move |stylesheet| {
            let mut failed = vec![];
            let okay = rules.find_map(|rule| {
                if let Ok(declaration) = bindings::make_rule(stylesheet, rule) {
                    Some(declaration)
                } else {
                    failed.push(String::from(rule));
                    None
                }
            });
            if let Some(okay) = okay {
                okay
            } else {
                panic!("selectors ar incorrect:\n {}", failed.join("\n "));
            }
        })
    }

    pub fn __internal_stylesheet<A>(rules: A) -> Self
    where
        A: MultiStr,
    {
        let element = Self::__internal_rules(&rules).unchecked_into::<CssStyleRule>();
        Self {
            element: element.style(),
            callbacks: Callbacks::new(),
        }
    }

    pub fn style<B, C>(self, name: B, value: C) -> Self
    where
        B: MultiStr,
        C: MultiStr,
    {
        set_style(&self.element, &name, value, false);
        self
    }

    pub fn style_important<B, C>(self, name: B, value: C) -> Self
    where
        B: MultiStr,
        C: MultiStr,
    {
        set_style(&self.element, &name, value, true);
        self
    }

    pub fn style_unchecked<B, C>(self, name: B, value: C) -> Self
    where
        B: AsStr,
        C: AsStr,
    {
        name.with_str(|name| {
            value.with_str(|value| {
                bindings::set_style(&self.element, intern(name), value, false);
            })
        });
        self
    }

    pub fn style_signal<B, C, D, E>(mut self, name: B, value: E) -> Self
    where
        B: MultiStr + 'static,
        C: MultiStr,
        D: OptionStr<Output = C>,
        E: Signal<Item = D> + 'static,
    {
        set_style_signal(
            self.element.clone(),
            &mut self.callbacks,
            name,
            value,
            false,
        );
        self
    }

    pub fn style_important_signal<B, C, D, E>(mut self, name: B, value: E) -> Self
    where
        B: MultiStr + 'static,
        C: MultiStr,
        D: OptionStr<Output = C>,
        E: Signal<Item = D> + 'static,
    {
        set_style_signal(self.element.clone(), &mut self.callbacks, name, value, true);
        self
    }

    pub fn style_unchecked_signal<B, C, D, E>(mut self, name: B, value: E) -> Self
    where
        B: AsStr + 'static,
        C: AsStr,
        D: OptionStr<Output = C>,
        E: Signal<Item = D> + 'static,
    {
        set_style_unchecked_signal(
            self.element.clone(),
            &mut self.callbacks,
            name,
            value,
            false,
        );
        self
    }

    pub fn raw<B>(self, css: B) -> Self
    where
        B: AsStr,
    {
        css.with_str(|css| {
            bindings::append_raw(&self.element, css);
        });
        self
    }

    pub fn __internal_done(mut self) {
        self.callbacks.trigger_after_insert();
        self.callbacks.leak();
    }
}

fn set_style_unchecked_signal<A, B, C, D>(
    style: CssStyleDeclaration,
    callbacks: &mut Callbacks,
    name: A,
    value: D,
    important: bool,
) where
    A: AsStr + 'static,
    B: AsStr,
    C: OptionStr<Output = B>,
    D: Signal<Item = C> + 'static,
{
    set_option(style, callbacks, value, move |style, value| match value {
        Some(value) => {
            name.with_str(|name| {
                let name: &str = intern(name);
                value.with_str(|value| {
                    bindings::set_style(style, name, value, important);
                });
            });
        }
        None => name.with_str(|name| {
            bindings::remove_style(style, intern(name));
        }),
    });
}

fn set_style_signal<A, B, C, D>(
    style: CssStyleDeclaration,
    callbacks: &mut Callbacks,
    name: A,
    value: D,
    important: bool,
) where
    A: MultiStr + 'static,
    B: MultiStr,
    C: OptionStr<Output = B>,
    D: Signal<Item = C> + 'static,
{
    set_option(style, callbacks, value, move |style, value| match value {
        Some(value) => {
            set_style(style, &name, value, important);
        }
        None => name.each(|name| {
            bindings::remove_style(style, intern(name));
        }),
    })
}

fn set_option<A, B, C, D, F>(element: A, callbacks: &mut Callbacks, value: D, mut f: F)
where
    A: 'static,
    C: OptionStr<Output = B>,
    D: Signal<Item = C> + 'static,
    F: FnMut(&A, Option<B>) + 'static,
{
    let mut is_set = false;
    callbacks.after_remove(for_each(value, move |value| {
        let value = value.into_option();
        if value.is_some() {
            is_set = true;
        } else if is_set {
            is_set = false;
        } else {
            return;
        }
        f(&element, value);
    }));
}
fn set_style<A, B>(style: &CssStyleDeclaration, name: &A, value: B, important: bool)
where
    A: MultiStr,
    B: MultiStr,
{
    let mut names = vec![];
    let mut values = vec![];

    fn try_set_style(
        style: &CssStyleDeclaration,
        names: &mut Vec<String>,
        values: &mut Vec<String>,
        name: &str,
        value: &str,
        important: bool,
    ) -> Option<()> {
        assert!(value != "");
        bindings::remove_style(style, name);
        bindings::set_style(style, name, value, important);
        let is_changed = bindings::get_style(style, name) != "";
        if is_changed {
            Some(())
        } else {
            names.push(String::from(name));
            values.push(String::from(value));
            None
        }
    }
    let okay = name.find_map(|name| {
        let name: &str = intern(name);
        value.find_map(|value| {
            try_set_style(style, &mut names, &mut values, &name, &value, important)
        })
    });

    if let None = okay {
        if cfg!(debug_assertions) {
            panic!(
                "style is incorrect:\n names: {}\n values: {}",
                names.join(", "),
                values.join(", ")
            );
        }
    }
}

fn create_element<A>(name: &str) -> A
where
    A: JsCast,
{
    crate::__unwrap!(
        bindings::create_element(intern(name)).dyn_into(),
        e => panic!("Invalid DOM type: \"{}\" => {:?}", name, JsValue::as_ref(&e)),
    )
}

fn create_element_ns<A>(name: &str, namespace: &str) -> A
where
    A: JsCast,
{
    crate::__unwrap!(
        bindings::create_element_ns(intern(namespace), intern(name)).dyn_into(),
        e => panic!("Invalid DOM type \"{}\" => {:?}", name, JsValue::as_ref(&e)),
    )
}

#[derive(Debug)]
pub struct DomBuilder<A> {
    element: A,
    callbacks: Callbacks,
}

impl<A> DomBuilder<A>
where
    A: JsCast,
{
    pub fn new_html(name: &str) -> Self {
        Self::new(create_element(name))
    }

    pub fn new_svg(name: &str) -> Self {
        Self::new(create_element_ns(name, SVG_NAMESPACE))
    }
}

impl<A> DomBuilder<A> {
    pub fn __internal_transfer_callbacks<B>(mut self, mut shadow: DomBuilder<B>) -> Self {
        self.callbacks
            .after_insert
            .append(&mut shadow.callbacks.after_insert);
        self.callbacks
            .after_remove
            .append(&mut shadow.callbacks.after_remove);
        self
    }

    pub fn new(value: A) -> Self {
        Self {
            element: value,
            callbacks: Callbacks::new(),
        }
    }
}
