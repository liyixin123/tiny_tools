use druid::{Data, Lens, Widget, WidgetExt, WindowDesc};
use druid::widget::{Button, Flex, ListIter, TextBox};
use druid::im::Vector;

#[derive(Clone, Data, Lens)]
struct TextBoxData {
    text: String,
}

#[derive(Clone, Data, Lens)]
struct AppData {
    text_boxes: Vector<TextBoxData>,
}

impl ListIter<TextBoxData> for AppData {
    fn for_each(&self, mut cb: impl FnMut(&TextBoxData, usize)) {
        self.text_boxes.iter().enumerate().for_each(|(index, item)| {
            cb(item, index);
        })
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut TextBoxData, usize)) {
        self.text_boxes.iter_mut().enumerate().for_each(|(index, item)| {
            cb(item, index);
        });
    }

    fn data_len(&self) -> usize {
        self.text_boxes.len()
    }
}

fn ui_builder() -> impl Widget<AppData> {
    let button = Button::new("Add TextBox")
        .on_click(|ctx, data: &mut AppData, _env| {
            data.text_boxes.push_back(
                TextBoxData {
                    text: data.text_boxes.len().to_string()
                });
            ctx.request_update()
        });

    let text_boxes = Flex::column()
        .with_child(button)
        .with_flex_child(
            druid::widget::List::new(|| {
                TextBox::new().expand_width().lens(TextBoxData::text)
            }),
            1.0,
        );

    Flex::column().with_child(text_boxes)
}

pub fn main() {
    let data = AppData {
        text_boxes: Vector::new(),
    };
    let main_window = WindowDesc::new(ui_builder())
        .title("标题");
    let _ = druid::AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(data)
        .expect("launch failed");
}