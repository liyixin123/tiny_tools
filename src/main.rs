// 目标：写一个程序，将平时的SVN地址格式转换一下

#![windows_subsystem = "windows"]

mod pic_uploader;
mod tests;

use anyhow::Result;
use druid::im::Vector;
use druid::text::{FontDescriptor, FontFamily};
use druid::widget::{Checkbox, CrossAxisAlignment, Flex, Label, ListIter, Tabs};
use druid::{
    widget::{Button, TextBox},
    AppLauncher, Application, Color, Data, Env, Lens, Widget, WidgetExt, WindowDesc,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::Write;
use std::sync::Mutex;
use subversion_edge_modify_tool::permissions::Permissions;
use subversion_edge_modify_tool::start_init::get_backups_dir;
use subversion_edge_modify_tool::{modify_auths_local, modify_auths_remote};

const BASE_URL: &str = "http://172.17.102.22:18080/svn/softwarerepo";
const BAD_FORMAT_STR: &str = "格式错误";
const BUTTON_WIDTH: f64 = 150.0;
lazy_static! {
    static ref SEPARATOR: Mutex<Vec<char>> = Mutex::new(vec!['/', ' ',]);
    static ref REGEXES: Vec<Regex> = vec![
        Regex::new(r"（.*?$").unwrap(),
        Regex::new(r"\(.*?$").unwrap(),
        Regex::new(r"、.*?$").unwrap(),
        Regex::new(r"，.*?$").unwrap(),
        Regex::new(r",.*?$").unwrap(),
    ];
}

#[derive(Clone, Data, Lens)]
struct TextBoxData {
    text: String,
}

impl ListIter<TextBoxData> for SVNAddress {
    fn for_each(&self, mut cb: impl FnMut(&TextBoxData, usize)) {
        self.new_addrs.iter().enumerate().for_each(|(index, item)| {
            cb(item, index);
        })
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut TextBoxData, usize)) {
        self.new_addrs
            .iter_mut()
            .enumerate()
            .for_each(|(index, item)| {
                cb(item, index);
            });
    }

    fn data_len(&self) -> usize {
        self.new_addrs.len()
    }
}

#[derive(Clone, Copy)]
enum MessageType {
    Info,
    Error,
}

#[derive(Data, Clone, Lens)]
struct SVNAddress {
    old: String,
    new_addrs: Vector<TextBoxData>,
    names: String,
    read_write: bool,
    backup_path: String,
    message: String,
    message_color: druid::Color,
}

impl SVNAddress {
    fn new() -> SVNAddress {
        SVNAddress {
            old: "".to_string(),
            new_addrs: Vector::new(),
            names: String::new(),
            read_write: true,
            backup_path: {
                let path = get_backups_dir().unwrap();
                path.to_str().unwrap().to_string()
            },
            message: "".to_string(),
            message_color: Color::GREEN,
        }
    }
    #[allow(dead_code)]
    fn set_old(&mut self, old:&str){
        self.old = old.to_string();
    }
    fn update(&mut self) {
        if let Some(name) = extract_name(self.old.as_str()) {
            self.names = name;
        }
        if self.names.is_empty() {
            self.set_message("请输入用户名".to_string(), MessageType::Info);
            return;
        }

        self.new_addrs.clear();
        let url_list = match extract_substrings_containing_base_url(self.old.as_str()) {
            Ok(urls) => urls,
            Err(e) => {
                let err = format!("Error:{}", e);
                self.set_message(err, MessageType::Error);
                return;
            }
        };
        // 提取名字

        // 获取权限
        if let Some(permissions) = extract_permissions(self.old.as_str()) {
            if permissions == "只读" {
                self.read_write = false;
            } else {
                // } else if permissions.contains("写"){
                self.read_write = true;
            }
        }

        if url_list.is_empty() || !url_list.first().unwrap().starts_with("http") {
            self.new_addrs.push_back(TextBoxData {
                text: BAD_FORMAT_STR.to_string(),
            });
            self.set_message(BAD_FORMAT_STR.to_string(), MessageType::Error);
            return;
        } else {
            self.set_message("地址检查成功".to_string(), MessageType::Info);
        }

        for x in url_list {
            self.new_addrs.push_back(TextBoxData {
                text: convert_address(x),
            });
        }
    }
    #[allow(dead_code)]
    fn merged_new_addr(&self) -> String {
        let mut builder = String::new();
        for textbox in &self.new_addrs {
            if !builder.is_empty() {
                builder.push('\n');
            }
            write!(builder, "{}", &textbox.text).expect("write failed");
        }
        builder
    }
    fn set_message(&mut self, message: String, message_type: MessageType) {
        self.message = message;
        match message_type {
            MessageType::Info => self.message_color = druid::Color::GREEN, // 绿色
            MessageType::Error => self.message_color = druid::Color::RED,  // 红色
        }
    }

    fn generate_permissions(&mut self) -> Option<Vec<Permissions>> {
        let mut result = Vec::new();
        let new_addrs = self.new_addrs.clone();
        for addr in &new_addrs {
            if &addr.text == BAD_FORMAT_STR || addr.text.is_empty() {
                self.set_message("权限生成失败".to_string(), MessageType::Error);
                return None;
            } else {
                let message = "权限转化成功".to_string();
                let message_type = MessageType::Info;
                self.set_message(message, message_type);
            }
            let repo = &addr.text;
            let auth = if self.read_write { "rw" } else { "r" };

            let names = split_string(&self.names);
            let mut users = Vec::new();
            for name in names {
                let user_auth = subversion_edge_modify_tool::permissions::UserAuth {
                    user: name.to_string(),
                    auth: String::from(auth),
                };
               users.push(user_auth);
            }
            // let permission = Permissions::new(repo, user, auth);
            let permission = Permissions::new_from_users(repo, users);
            result.push(permission);
        }

        Some(result)
    }
    async fn apply_to_local(&mut self) {
        if let Some(permissions) = self.generate_permissions() {
            modify_auths_local(&permissions).await;
        }
    }
    async fn apply_to_remote(&mut self) {
        if let Some(permissions) = self.generate_permissions() {
            modify_auths_remote(&permissions).await;
        }
    }
}

fn extract_substrings_containing_base_url(input_str: &str) -> Result<Vec<String>> {
    let result: Vec<String> = input_str
        .split_whitespace()
        .filter_map(|s| s.find(BASE_URL).map(|start| s[start..].to_string()))
        .collect();
    if result.is_empty() {
        Err(anyhow::anyhow!("URL格式不符合要求：{} ...", BASE_URL))
    } else {
        Ok(result)
    }
}

fn extract_name(input_str: &str) -> Option<String> {
    let account_name_re = Regex::new(r"SVN账号名称\r?\n(.+?)(?:\r?\n|\r?$)").unwrap();
    let account_name = account_name_re
        .captures(input_str)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()));
    account_name
}

fn extract_permissions(input_str: &str) -> Option<String> {
    let re = Regex::new(r"(只读|读写|读|写)").unwrap();
    let permission = input_str
        .lines()
        .rev() // 从末尾开始查找
        .find(|line| re.is_match(line))
        .map(|m| re.find(m).unwrap().as_str().trim().to_string());

    permission
}

fn split_string(input: &str) -> Vec<&str> {
    input.split([',', '，','、', ';','；'].as_ref())
    .map(|s| s.trim())
    .collect()
}
fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title("小工具")
        .window_size((1200.0, 800.0));
    let initial_state = SVNAddress::new();

    AppLauncher::with_window(main_window)
        .log_to_console()
        .configure_env(|env, _| {
            let font_name = "阿里巴巴普惠体 3.0";
            if is_font_installed(&font_name) {
                let font =
                    FontDescriptor::new(FontFamily::new_unchecked(font_name)).with_size(16.0);
                env.set(druid::theme::UI_FONT, font);
            }
        })
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn is_separator(c: char) -> bool {
    SEPARATOR.lock().unwrap().contains(&c)
}

fn replace_str(src: String) -> String {
    let ret = src
        .replace(BASE_URL, "softwarerepo:")
        .trim_end_matches(|c| is_separator(c))
        .to_string();

    REGEXES
        .iter()
        .fold(ret, |acc, regex| regex.replace_all(&acc, "").to_string())
}

fn convert_address(src: String) -> String {
    let mut ret;
    if src.contains(BASE_URL) {
        ret = replace_str(src);
        ret = format!("[{}]", ret);
    } else {
        ret = BAD_FORMAT_STR.to_owned()
    }
    ret
}

fn open_folder(path: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(path).spawn()?;
    }
    Ok(())
}
fn is_font_installed(font_name: &str) -> bool {
    font_kit::source::SystemSource::new()
        .select_family_by_name(font_name)
        .is_ok()
}
fn build_root_widget() -> impl Widget<SVNAddress> {
    let label_svn = Label::dynamic(|data: &SVNAddress, _env: &Env| {
        if data.message.is_empty() {
            "输入原始地址，点击结果框可直接复制".to_string()
        } else {
            data.message.clone()
        }
    })
    .with_text_size(32.0)
    .env_scope(|env, data| env.set(druid::theme::TEXT_COLOR, data.message_color.clone()));

    let textbox = TextBox::multiline()
        .with_placeholder("原始地址")
        .expand_width()
        .lens(SVNAddress::old)
        .expand_height();
    let textbox_name = TextBox::new()
        .with_placeholder("用户名")
        .lens(SVNAddress::names);
    // let name_list = COm

    let textbox_out = Flex::column()
        .with_flex_child(
            druid::widget::List::new(|| {
                TextBox::new()
                    .expand_width()
                    .lens(TextBoxData::text)
                    .on_click(|_ctx, data, _env| {
                        let mut clipboard = Application::global().clipboard();
                        clipboard.put_string(&data.text);
                    })
            }),
            1.0,
        )
        .expand_height();

    let btn_process = Button::<SVNAddress>::new("转换")
        .fix_width(BUTTON_WIDTH)
        .on_click(|_ctx, _data, _env| {
            _data.update();
            _ctx.request_paint();
        });
    let btn_open_url = Button::<SVNAddress>::new("打开网页")
        .fix_width(BUTTON_WIDTH)
        .on_click(|_ctx, _data, _env| {
            if let Err(e) = open::with(
                "http://172.17.102.22:3343/csvn/repo/editAuthorization?",
                "chrome",
            ) {
                eprintln!("Failed to open URL: {}", e);
            }
        });
    let btn_save_local = Button::<SVNAddress>::new("保存本地")
        .fix_width(BUTTON_WIDTH)
        .disabled_if(|data, _| {
            data.names.is_empty()
                || data.old.is_empty()
                || data.new_addrs.is_empty()
                || data.message == BAD_FORMAT_STR
        })
        .on_click(|_ctx, data, _env| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(data.apply_to_local());
            data.set_message("权限生成成功,打开备份查看".to_string(), MessageType::Info);
        });
    let btn_open_backup = Button::<SVNAddress>::new("查看备份")
        .fix_width(BUTTON_WIDTH)
        .on_click(|_, data, _| {
            open_folder(data.backup_path.as_str()).unwrap();
        });
    let btn_apply_to_remote = Button::<SVNAddress>::new("应用到服务器")
        .fix_width(BUTTON_WIDTH)
        .disabled_if(|data, _| {
            data.names.is_empty()
                || data.old.is_empty()
                || data.new_addrs.is_empty()
                || data.message == BAD_FORMAT_STR
        })
        .on_click(|_ctx, data, _env| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            data.set_message("正在修改权限...".to_string(), MessageType::Info);
            rt.block_on(data.apply_to_remote());
            data.set_message("权限生成成功".to_string(), MessageType::Info);
        });
    let checkbox_read_write = Checkbox::new("读写").lens(SVNAddress::read_write);
    let mut col = Flex::column().with_flex_child(label_svn, 1.0);

    col.add_flex_child(textbox, 3.0);
    col.add_flex_child(textbox_name.center(), 1.0);
    col.add_flex_child(checkbox_read_write.center(), 1.0);
    col.add_flex_child(btn_process, 1.0);
    col.add_flex_child(btn_open_url.align_right(), 1.0);
    col.add_flex_child(btn_save_local.align_right(), 1.0);
    col.add_flex_child(btn_open_backup.align_right(), 1.0);
    col.add_flex_child(btn_apply_to_remote, 1.0);
    col.add_flex_child(textbox_out, 5.0);
    col.set_cross_axis_alignment(CrossAxisAlignment::Center);

    let tabs = Tabs::new()
        .with_tab("SVN权限开通", col)
        .with_tab("Proxy", Label::new("Proxy settings"));

    Flex::column().with_flex_child(tabs, 1.0)
    // .debug_paint_layout()
}
