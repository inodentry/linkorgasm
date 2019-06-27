use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;

use cursive::traits::*;
use cursive::views::{OnEventView, Dialog, EditView, SelectView, LinearLayout, DummyView, ScrollView};
use cursive::Cursive;

use hashbrown::{HashMap, HashSet};

#[derive(Debug)]
struct Item {
    name: String,
    path: PathBuf,
}

#[derive(Debug)]
struct Tag {
    name: String,
    path: PathBuf,
    items: HashMap<PathBuf, PathBuf>,
}

#[derive(Debug)]
struct AppState {
    items: HashMap<PathBuf, Item>,
    tags: HashMap<PathBuf, Tag>,
    tags_path: PathBuf,
    sel: HashSet<PathBuf>,
}

fn scan_items(state: &mut AppState, p: impl AsRef<Path>) {
    for entry in fs::read_dir(p).expect("cannot access all dir") {
        let entry = entry.expect("error scanning all dir");
        let path = entry.path();
        let cpath = path.canonicalize().unwrap();
        state.items.insert(
            cpath,
            Item {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path,
            }
        );
    }
}

fn scan_tags(state: &mut AppState, mut parent: Option<&mut Tag>, p: impl AsRef<Path>) {
    let p = p.as_ref();
    for entry in fs::read_dir(p).expect("cannot access tags dir") {
        let entry = entry.expect("error scanning tags dir");
        let path = entry.path();
        let cpath = path.canonicalize().unwrap();
        if path.is_dir() {
            let mut tag = Tag {
                name: path.strip_prefix(&state.tags_path).unwrap()
                    .to_string_lossy().to_string(),
                path: path.to_owned(),
                items: HashMap::default(),
            };
            scan_tags(state, Some(&mut tag), &path);
            state.tags.insert(cpath, tag);
        } else if state.items.contains_key(&cpath) {
            if let Some(ref mut parent) = parent {
                parent.items.insert(cpath, path);  
            }
        }
    }
}

fn ui_refresh_itemview(siv: &mut Cursive) {
    let mut state = siv.take_user_data::<AppState>().unwrap();
    state.sel.clear();

    siv.call_on_id("itemview", |v: &mut SelectView<PathBuf>| {
        v.clear();
        for (p, i) in state.items.iter() {
            v.add_item(i.name.clone(), p.clone());
        }
        v.sort_by_label();
    });

    siv.set_user_data(state);

    ui_mark_itemview(siv);
    ui_refresh_tagsview(siv);
}

fn ui_refresh_tagsview(siv: &mut Cursive) {
    let state = siv.take_user_data::<AppState>().unwrap();

    siv.call_on_id("tagsview", |v: &mut SelectView<PathBuf>| {
        v.clear();
        for (p, t) in state.tags.iter() {
            v.add_item(t.name.clone(), p.clone());
        }
        v.sort_by_label();
    });

    siv.set_user_data(state);

    ui_mark_tagsview(siv);
}

fn ui_mark_itemview(siv: &mut Cursive) {
    let state = siv.take_user_data::<AppState>().unwrap();

    siv.call_on_id("itemview", |v: &mut SelectView<PathBuf>| {
        for i in 0..v.len() {
            let (s, p) = v.get_item_mut(i).unwrap();
            let item = state.items.get(p).unwrap();

            *s = format!(
                "{} {}",
                if state.sel.contains(p) {
                    "[X]"
                } else {
                    "[ ]"
                },
                item.name
            ).into();
        }
    });

    siv.set_user_data(state);
}

fn ui_mark_tagsview(siv: &mut Cursive) {
    let state = siv.take_user_data::<AppState>().unwrap();

    siv.call_on_id("tagsview", |v: &mut SelectView<PathBuf>| {
        for i in 0..v.len() {
            let (s, p) = v.get_item_mut(i).unwrap();
            let t = state.tags.get(p).unwrap();

            let mut oncount = 0;
            let mut offcount = 0;

            for item in state.sel.iter() {
                if t.items.contains_key(item) {
                    oncount += 1;
                } else {
                    offcount += 1;
                }
            }

            *s = format!(
                "{} {}",
                match (oncount, offcount) {
                    (0, _) => "[ ]",
                    (_, 0) => "[X]",
                    (_, _) => "[?]",
                },
                t.name
            ).into();
        }
    });

    siv.set_user_data(state);
}

fn tag_path_prefix(tag: &Path, item: &Path) -> PathBuf {
    // assuming both input paths are canonical
    let mut tc = tag.components();
    let mut ic = item.components();

    while tc.next() == ic.next() {}

    let count = tc.count();

    let mut p = PathBuf::from("../");
    for _ in 0..count {
        p.push("../");
    }
    p
}

fn toggle_sel(siv: &mut Cursive) {
    let p = siv.call_on_id("itemview", |v: &mut SelectView<PathBuf>| {
        v.selection().unwrap()
    }).unwrap();
    let state = siv.user_data::<AppState>().unwrap();
    if state.sel.contains(p.as_ref()) {
        state.sel.remove(p.as_ref());
    } else {
        state.sel.insert(p.to_path_buf());
    }
}

fn toggle_tag(siv: &mut Cursive) {
    let mut state = siv.take_user_data::<AppState>().unwrap();
    let tp = siv.call_on_id("tagsview", |v: &mut SelectView<PathBuf>| {
        v.selection().unwrap()
    }).unwrap();
    let tag = state.tags.get_mut(tp.as_path()).unwrap();
    for ip in state.sel.iter() {
        if let Some(p) = tag.items.get(ip) {
            fs::remove_file(p).expect("could not delete symlink");
            tag.items.remove(ip);
        } else {
            let item = state.items.get(ip).unwrap();
            let mut target = tag_path_prefix(&tp, &ip);
            target.push(&item.path);
            let mut link = tp.to_path_buf();
            link.push(&item.name);

            std::os::unix::fs::symlink(&target, &link)
                .expect("could not create symlink");
            tag.items.insert(ip.to_owned(), link);
        }
    }
    siv.set_user_data(state);
}

fn ui_cmdexec(siv: &mut Cursive, cmd: &str) {
    let state = siv.take_user_data::<AppState>().unwrap();
    siv.pop_layer();
    for item in state.sel.iter() {
        let mut cmd = Command::new(cmd);
        cmd.args(&[item]);
        if let Err(e) = cmd.spawn() {
            siv.add_layer(
                Dialog::text(format!("{}", e))
                    .title("ERROR")
                    .button("Ok", |siv| { siv.pop_layer(); })
            )
        }
    }
    siv.set_user_data(state);
}

fn ui_build_cmdexec(siv: &mut Cursive) {
    siv.add_layer(
        ui_input_dialog("Open selection with:", "cmd", "", ui_cmdexec)
    );
}

fn ui_do_new_tag(siv: &mut Cursive, name: &str) {
    if !name.is_empty() {
        siv.pop_layer();

        {
            let state = siv.user_data::<AppState>().unwrap();

            let mut path = state.tags_path.clone();
            path.push(name);

            dbg!(&path);

            fs::DirBuilder::new()
                .recursive(true)
                .create(&path)
                .expect("could not create dir");

            state
                .tags.insert(
                    path.canonicalize().unwrap(),
                    Tag {
                        name: name.to_owned(),
                        path: path,
                        items: HashMap::default(),
                    }
                );
        }

        ui_refresh_tagsview(siv);
    }
}

fn ui_build_new_tag(siv: &mut Cursive) {
    siv.add_layer(
        ui_input_dialog("New tag:", "tagname", "", ui_do_new_tag)
    );
}

fn ui_build_main(siv: &mut Cursive) {
    let itemview = SelectView::<PathBuf>::new()
        .with_id("itemview");
    let itemview = OnEventView::new(itemview)
        .on_event(' ', |siv| {
            toggle_sel(siv);
            ui_mark_itemview(siv);
            ui_mark_tagsview(siv);
        })
        .on_event('e', ui_build_cmdexec);
    let itemview = ScrollView::new(itemview)
        .scroll_x(true);

    let tagsview = SelectView::<PathBuf>::new()
        .with_id("tagsview");
    let tagsview = OnEventView::new(tagsview)
        .on_event(' ', |siv| {
            toggle_tag(siv);
            ui_mark_tagsview(siv);
        })
        .on_event('+', ui_build_new_tag);
    let tagsview = ScrollView::new(tagsview);

    let layout = LinearLayout::horizontal()
        .child(itemview)
        .child(DummyView)
        .child(tagsview);

    siv.add_layer(Dialog::around(layout).title("linkorgasm"));

    ui_refresh_itemview(siv);
}

fn ui_submit_itemdir(siv: &mut Cursive, p: &str) {
    let state = siv.user_data().unwrap();
    scan_items(state, p);
    siv.pop_layer();
    siv.add_layer(
        ui_input_dialog("Tags directory:", "tagdir", "tags", ui_submit_tagdir)
    );
}

fn ui_submit_tagdir(siv: &mut Cursive, p: &str) {
    let mut state = siv.user_data::<AppState>().unwrap();
    state.tags_path = PathBuf::from(p);
    scan_tags(state, None, p);
    siv.pop_layer();
    ui_build_main(siv);
}

fn ui_input_dialog(
    title: &str,
    id: &'static str,
    default: &str,
    submit: fn(&mut Cursive, &str)
) -> Dialog {
    Dialog::new()
        .title(title)
        .content(
            EditView::new()
                .on_submit(submit)
                .content(default)
                .with_id(id)
                .fixed_width(20)
        )
        .button("Ok", move |siv| {
            let text = siv.call_on_id(id, |v: &mut EditView| {
                v.get_content()
            }).unwrap();
            submit(siv, &text);
        })
}

fn main() {
    let mut siv = Cursive::default();

    siv.set_user_data(AppState {
        items: HashMap::default(),
        tags: HashMap::default(),
        tags_path: PathBuf::new(),
        sel: HashSet::default(),
    });
    siv.add_global_callback('q', |siv| siv.quit());

    siv.add_layer(
        ui_input_dialog("Items directory:", "itemdir", "all", ui_submit_itemdir)
    );

    siv.run();
}

