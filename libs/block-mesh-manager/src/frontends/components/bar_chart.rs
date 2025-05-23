use crate::frontends::context::size_context::SizeContext;
use block_mesh_common::interfaces::server_api::DashboardResponse;
use charming::component::{Grid, Title};
use charming::datatype::CompositeValue;
use charming::element::{
    AxisLabel, AxisPointer, AxisPointerType, Color, Emphasis, ItemStyle, TextStyle, Tooltip,
    Trigger,
};
use charming::{component::Axis, element::AxisType, series::Bar, Chart, HtmlRenderer};
use leptos::*;

#[component]
pub fn BarChart() -> impl IntoView {
    let async_data = use_context::<DashboardResponse>();
    let size_context = use_context::<SizeContext>().unwrap();
    let daily_stats = RwSignal::new(vec![]);
    if let Some(data) = async_data {
        daily_stats.set(data.daily_stats)
    }

    let width = Signal::derive(move || {
        let w = size_context.width.get();
        w * 0.5
    });

    let html_chart =
        Signal::derive({
            let data = daily_stats.get_untracked();
            move || {
                let chart =
                    Chart::new()
                        .title(Title::new().text("Daily Points").left("center").text_style(
                            TextStyle::new().font_size(20).color(Color::from("#f9f9f9")),
                        ))
                        .grid(
                            Grid::new()
                                .contain_label(true)
                                .left(CompositeValue::String("3%".to_string()))
                                .right(CompositeValue::String("4%".to_string()))
                                .bottom(CompositeValue::String("3%".to_string()))
                                .background_color("#f9f9f9"),
                        )
                        .x_axis(
                            Axis::new()
                                .type_(AxisType::Category)
                                .axis_label(
                                    AxisLabel::new()
                                        .show(true)
                                        .rotate(45)
                                        .color(Color::from("#f9f9f9")),
                                )
                                .data(data.iter().map(|i| i.day.to_string()).collect()),
                        )
                        .y_axis(
                            Axis::new()
                                .type_(AxisType::Value)
                                .axis_label(AxisLabel::new().color(Color::from("#f9f9f9"))),
                        )
                        .axis_pointer(AxisPointer::new().type_(AxisPointerType::Shadow))
                        .tooltip(
                            Tooltip::new()
                                .trigger(Trigger::Axis)
                                .background_color("#ffffff")
                                .border_color("#ccc")
                                .border_width(1)
                                .padding(10),
                        )
                        .series(
                            Bar::new()
                                .bar_width(40)
                                .name("Points")
                                .data(data.iter().map(|i| i.points).collect())
                                .item_style(
                                    ItemStyle::new()
                                        .color(Color::from("#DA00DB")) // Pleasant green color
                                        .border_radius(5.0)
                                        .opacity(0.9),
                                )
                                .emphasis(
                                    Emphasis::new().item_style(
                                        ItemStyle::new()
                                            .color(Color::from("#45a049"))
                                            .shadow_blur(10)
                                            .shadow_color("#333")
                                            .opacity(1.0), // Full opacity on hover
                                    ),
                                ),
                        );
                let html_renderer = HtmlRenderer::new("Daily Points", width.get() as u64, 400);
                let res = html_renderer.render(&chart);

                res.unwrap_or_default()
            }
        });

    view! {
        <div class="flex justify-center items-center mt-4 m-2 relative overflow-hidden rounded-[30px] pt-6 md:pt-[33px] pb-7 md:pb-[39px] pl-[11px] md:pl-[44px]">
            <div class="m-2 grid grid-cols-1">
                <iframe
                    srcdoc=move || html_chart.get()
                    width=move || width.get()
                    height="450"
                ></iframe>
            </div>
        </div>
    }
}
