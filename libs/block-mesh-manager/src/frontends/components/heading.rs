use leptos::*;

#[component]
pub fn Heading(children: Children) -> impl IntoView {
    view! { <h1 class="font-bebas-neue text-2xl/8 font-semibold text-off-white">{children()}</h1> }
}
