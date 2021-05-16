use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use druid::{
    widget::{Button, Flex, Label, List, Scroll, TextBox},
    AppDelegate, Env, FileDialogOptions, WidgetExt,
};
use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathData(PathBuf);

impl AsRef<Path> for PathData {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl druid::Data for PathData {
    fn same(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, druid::Lens, druid::Data)]
struct AppData {
    items: Arc<Vec<PathData>>,
    git_path: PathData,
    input: String,
}

fn load_items<P, Keep>(path: P, filter: Keep) -> Vec<PathData>
where
    P: AsRef<Path>,
    Keep: Fn(&Path) -> bool,
{
    let path = path.as_ref();
    std::fs::read_dir(path)
        .expect("Unable to read path")
        .map(|entry| {
            let entry = entry.expect("Unable to fetch an entry");
            entry.path()
        })
        .filter(|x| filter(x))
        .map(PathData)
        .collect()
}

struct FilteredItems;

impl FilteredItems {
    fn filter(data: &AppData) -> Arc<Vec<PathData>> {
        Arc::new(load_items(&data.git_path, |path| {
            path.display().to_string().contains(&data.input)
        }))
    }
}

impl druid::Lens<AppData, Arc<Vec<PathData>>> for FilteredItems {
    fn with<V, F: FnOnce(&Arc<Vec<PathData>>) -> V>(&self, data: &AppData, f: F) -> V {
        let items = Self::filter(data);
        f(&items)
    }

    fn with_mut<V, F: FnOnce(&mut Arc<Vec<PathData>>) -> V>(&self, data: &mut AppData, f: F) -> V {
        let mut items = Self::filter(data);
        f(&mut items)
    }
}

fn build_ui() -> impl Widget<AppData> {
    let mut root = Flex::column();

    let select_button =
        Button::new("Select path").on_click(|event_ctx, _data: &mut AppData, _env: &Env| {
            event_ctx.submit_command(
                druid::commands::SHOW_OPEN_PANEL
                    .with(FileDialogOptions::new().select_directories().show_hidden()),
            );
        });
    let reload_button = Button::new("Reload").on_click(|_event_ctx, data: &mut AppData, _env| {
        let items = Arc::new(load_items(&data.git_path, |path| {
            path.display().to_string().contains(&data.input)
        }));
        data.items = items;
    });

    let mut buttons = Flex::row();
    buttons.add_child(select_button);
    buttons.add_child(reload_button);

    root.add_child(buttons);

    let mut input_box = TextBox::new();
    input_box.set_placeholder("filter");
    root.add_child(input_box.lens(AppData::input).expand_width());

    let list = Scroll::new(
        List::new(|| Label::new(|data: &PathData, _env: &Env| data.0.display().to_string()))
            .with_spacing(5.0)
            .lens(FilteredItems),
    )
    .expand_width();

    root.add_flex_child(list, 1.0);

    root
}

struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppData,
        _env: &Env,
    ) -> druid::Handled {
        if let Some(file_info) = cmd.get(druid::commands::OPEN_FILE) {
            let git_path = PathData(file_info.path().to_owned());
            data.git_path = git_path;
            return druid::Handled::Yes;
        }
        druid::Handled::No
    }
}

fn main() -> Result<(), PlatformError> {
    let git_path = PathData("/home/mexus/.password-store".into());
    AppLauncher::with_window(WindowDesc::new(build_ui))
        .delegate(Delegate)
        .launch(AppData {
            items: Arc::new(load_items(&git_path, |_| true)),
            git_path,
            input: String::new(),
        })?;
    Ok(())
}
