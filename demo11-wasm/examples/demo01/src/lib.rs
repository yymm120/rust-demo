use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm");
}

struct App {
    counter: Mutable<i32>,
}

impl App {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            counter: Mutable::new(0),
        })
    }

    fn render(state: &Arc<Self>) -> Dom {
        static ROOT_CLASS: Lazy<String> = Lazy::new(|| class! {
            .style("display", "inline-block")
            .style("background-color", "black")
            .style("padding", "10px")
        });

        static TEXT_CLASS: Lazy<String> = Lazy::new(|| class! {
            .style("color", "white")
            .style("font-weight", "bold")
        });

        static BUTTON_CLASS: Lazy<String> = Lazy::new(|| class! {
            .style("display", "block")
            .style("width", "100px")
            .style("margin", "5px")
        });

        html!("div", {
            .class(&*ROOT_CLASS)
            .children(&mut [
                html!("div", {
                    .class(&*TEXT_CLASS)
                    .text_signal(state.counter.signal().map(|x| format!("Counter: {}", x)))
                }),
                html!("button", {
                    .class_

                })
            ])
        })
    }
}
