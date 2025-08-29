use web_sys::{Comment, CssRule, CssStyleDeclaration, CssStyleSheet, Document, DomTokenList, Element, History, HtmlElement, HtmlStyleElement, Node, Text, Window};
use js_sys::Reflect;
use wasm_bindgen::{intern, JsCast, JsValue, UnwrapThrowExt};
use crate::utils::UnwrapJsExt;



/// `set_property(obj, name, value)` 函数用于设置属性.
#[track_caller]
pub(crate) fn set_property(obj: &JsValue, name: &str, value: &JsValue) {
    Reflect::set(obj, &JsValue::from(name), value).unwrap_js();
}


thread_local! {
    /// `window`, `document`和`history`变量在每个线程中独享.
    /// 通过`with`方法来访问变量确保了线程中进行安全访问, 因为它实现了`Drop` trait.
    pub static WINDOW: Window = web_sys::window().unwrap_throw();
    static DOCUMENT: Document = WINDOW.with(|w| w.document().unwrap_throw());
    static HISTORY: History = WINDOW.with(|w| w.history().unwrap_js());
}

pub(crate) fn body() -> HtmlElement {
    DOCUMENT.with(|d| d.body().unwrap_throw())
}

pub(crate) fn ready_state() -> String {
    DOCUMENT.with(|d| d.ready_state())
}

pub(crate) fn current_url() -> String {
    WINDOW.with(|w| w.location().href().unwrap_js())
} 

pub(crate) fn go_to_url(url: &str) {
    HISTORY.with(|h| {
        h.push_state_with_url(&JsValue::NULL, "", Some(url)).unwrap_js();
    });
}

pub(crate) fn replace_url(url: &str) {
    HISTORY.with(|h| {
        h.replace_state_with_url(&JsValue::NULL, "", Some(url)).unwrap_js();
    });
}

pub(crate) fn make_rule(sheet: &CssStyleSheet, rule: &str) -> Result<CssRule, JsValue> {
    let rules = sheet.css_rules().unwrap_js();
    let length = rules.length();
    sheet.insert_rule_with_index(rule, length);
    Ok(rules.get(length).unwrap_throw())
}

pub(crate) fn get_element_by_id(id: &str) -> Element {
    DOCUMENT.with(|d| d.get_element_by_id(id).unwrap_throw())
}

pub(crate) fn create_element(name: &str) -> Element {
    DOCUMENT.with(|d| d.create_element(name).unwrap_js())
}

pub(crate) fn create_element_ns(namespace: &str, name: &str) -> Element {
    DOCUMENT.with(|d| d.create_element_ns(Some(namespace), name).unwrap_js())
}

pub(crate) fn create_text_node(value: &str) -> Text {
    DOCUMENT.with(|d| d.create_text_node(value))
}

pub(crate) fn set_text(elem: &Text, value: &str) {
    elem.set_data(value);
}

pub(crate) fn create_comment(value: &str) -> Comment {
    DOCUMENT.with(|d| d.create_comment(value))
}

pub(crate) fn create_empty_node() -> Node {
    create_comment(intern("")).into()
}

pub(crate) fn set_attribute(element: &Element, key: &str, value: &str) {
    element.set_attribute(key, value).unwrap_js();
}

pub(crate) fn set_attribute_ns(element: &Element, namespace: &str, key: &str, value: &str) {
    element.set_attribute_ns(Some(namespace), key, value).unwrap_js();
}

pub(crate) fn remove_attribute(element: &Element, key: &str) {
    element.remove_attribute(key).unwrap_js();
}

pub(crate) fn remove_attribute_ns(element: &Element, namespace: &str, key: &str) {
    element.remove_attribute_ns(Some(namespace), key).unwrap_js();
}

pub(crate) fn add_class(classes: &DomTokenList, value: &str) {
    classes.add_1(value).unwrap_js();
}

pub(crate) fn remove_class(class: &DomTokenList, value: &str)  {
    class.remove_1(value).unwrap_js();
} 

pub(crate) fn get_style(style: &CssStyleDeclaration, name: &str) -> String {
    style.get_property_value(name).unwrap_js()
}

pub(crate) fn remove_style(style: &CssStyleDeclaration, name: &str) {
    style.remove_property(name).unwrap_js();
}

pub(crate) fn set_style(style: &CssStyleDeclaration, name: &str, value: &str, important: bool) {
    let priority = if important { intern("important")} else { intern("")};
    style.set_property_with_priority(name, value, priority).unwrap_js();
}

pub(crate) fn append_raw(style: &CssStyleDeclaration, css: &str) {
    style.set_css_text(&(style.css_text() + css));
}

pub(crate) fn insert_child_before(parent: &Node, child: &Node, other: &Node) {
    parent.insert_before(child, Some(other)).unwrap_js();
}

pub(crate) fn replace_child(parent: &Node, new: &Node, old: &Node) {
    parent.replace_child(new, old).unwrap_js();
}

pub(crate) fn remove_child(parent: &Node, child: &Node) {
    parent.remove_child(child).unwrap_js();
}

pub(crate) fn focus(element: &HtmlElement) {
    element.focus().unwrap_js();
}

pub(crate) fn blur(element: &HtmlElement) {
    element.blur().unwrap_js();
}

/// create_stylesheet(css) 函数通过`str`创建`CssStyleSheet`.
pub(crate) fn create_stylesheet(css: Option<&str>) -> CssStyleSheet {
    DOCUMENT.with(|document| {
        let e: HtmlStyleElement = document.create_element("style").unwrap_js().unchecked_into();
        e.set_type("text/css");
        if let Some(css) = css {
            e.set_text_content(Some(css));
        }
        append_child(&document.head().unwrap_throw(), &e);
        e.sheet().unwrap_throw().unchecked_into()
    })
}

/// append_child(parent, child) 函数添加子节点.
pub(crate) fn append_child(parent: &Node, child: &Node) {
    parent.append_child(child).unwrap_js();
}
