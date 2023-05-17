use crate::error::Result;
use crate::handler::{HtmlView, log_error, render};
use crate::view::frontend::index::Index;


pub async fn index() -> Result<HtmlView> {
    let handler_name = "backend/index/index";
    let tmpl = Index{};
    render(tmpl).map_err(log_error(handler_name))
}