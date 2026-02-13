use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::SystemTime;

use iced::widget;
use iced::{Color, Element, Length, Padding, Subscription};
use iced_layout_core::Node;

fn build_padding(p: &iced_layout_core::Padding) -> Option<Padding> {
    if p.top.is_none() && p.right.is_none() && p.bottom.is_none() && p.left.is_none() {
        return None;
    }
    let mut pad = Padding::ZERO;
    if let Some(v) = p.top { pad.top = v; }
    if let Some(v) = p.right { pad.right = v; }
    if let Some(v) = p.bottom { pad.bottom = v; }
    if let Some(v) = p.left { pad.left = v; }
    Some(pad)
}

fn build_length(l: &iced_layout_core::Length) -> Length {
    match l {
        iced_layout_core::Length::Fill => Length::Fill,
        iced_layout_core::Length::FillPortion(v) => Length::FillPortion(*v),
        iced_layout_core::Length::Shrink => Length::Shrink,
        iced_layout_core::Length::Fixed(v) => Length::Fixed(*v),
    }
}

fn build_color(c: &iced_layout_core::Color) -> Color {
    Color { r: c.r, g: c.g, b: c.b, a: c.a }
}

fn build<'a, Message: 'a>(node: &Node) -> Element<'a, Message> {
    match node {
        Node::Text { content, attrs } => {
            let mut t = widget::text(content.clone());
            if let Some(size) = attrs.size {
                t = t.size(size);
            }
            if let Some(ref w) = attrs.width {
                t = t.width(build_length(w));
            }
            if let Some(ref h) = attrs.height {
                t = t.height(build_length(h));
            }
            if let Some(ref c) = attrs.color {
                t = t.color(build_color(c));
            }
            t.into()
        }
        Node::Container { id, padding, children } => {
            assert_eq!(
                children.len(),
                1,
                "<container> must have exactly 1 child element, found {}",
                children.len()
            );
            let child = build::<Message>(&children[0]);
            let mut c = widget::container(child);
            if let Some(pad) = build_padding(padding) {
                c = c.padding(pad);
            }
            if let Some(id_val) = id {
                c.id(id_val.clone()).into()
            } else {
                c.into()
            }
        }
    }
}

struct CachedLayout {
    mtime: SystemTime,
    root: Node,
}

struct Cache {
    entries: HashMap<String, CachedLayout>,
    paths_changed: bool,
}

static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(|| {
    Mutex::new(Cache {
        entries: HashMap::new(),
        paths_changed: false,
    })
});

/// Loads an XML layout file at runtime and returns an iced `Element`.
///
/// On the first call for a given path the file is read and parsed. Subsequent
/// calls return the cached widget tree unless the file's mtime has changed.
///
/// Paths are resolved relative to the current working directory.
pub fn hot_layout<'a, Message: 'a>(path: &str) -> Element<'a, Message> {
    let mut cache = CACHE.lock().unwrap();

    let needs_reload = match cache.entries.get(path) {
        None => true,
        Some(cached) => {
            let mtime = fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            mtime != cached.mtime
        }
    };

    if needs_reload {
        let xml = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
        let mtime = fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let root = iced_layout_xml::parse(&xml);
        let is_new = !cache.entries.contains_key(path);
        cache.entries.insert(
            path.to_owned(),
            CachedLayout { mtime, root },
        );
        if is_new {
            cache.paths_changed = true;
        }
    }

    let cached = cache.entries.get(path).unwrap();
    build(&cached.root)
}

/// Returns a subscription that watches all paths registered via [`hot_layout`]
/// for file changes. Emits `()` whenever a watched file is modified.
///
/// Map the output to your own message type:
/// ```ignore
/// iced_layout::hot_reload_subscription().map(|_| Message::LayoutChanged)
/// ```
pub fn hot_reload_subscription() -> Subscription<()> {
    Subscription::run(watch_stream)
}

fn watch_stream() -> impl iced::futures::Stream<Item = ()> {
    iced::stream::channel(64, |mut output: iced::futures::channel::mpsc::Sender<()>| async move {
        use iced::futures::SinkExt;
        use notify::{RecursiveMode, Watcher};

        let (tx, mut rx) = iced::futures::channel::mpsc::channel(64);

        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(_event) = res {
                let _ = tx.clone().try_send(());
            }
        })
        .expect("failed to create file watcher");

        let mut watched: std::collections::HashSet<String> = std::collections::HashSet::new();

        loop {
            {
                let mut cache = CACHE.lock().unwrap();
                if cache.paths_changed {
                    for path in cache.entries.keys() {
                        if !watched.contains(path) {
                            let p = Path::new(path);
                            if p.exists() {
                                let _ = watcher.watch(p, RecursiveMode::NonRecursive);
                                watched.insert(path.clone());
                            }
                        }
                    }
                    cache.paths_changed = false;
                }
            }

            let timeout = iced::futures::FutureExt::fuse(
                async_io::Timer::after(std::time::Duration::from_millis(500)),
            );
            let event = iced::futures::FutureExt::fuse(
                iced::futures::StreamExt::select_next_some(&mut rx),
            );

            iced::futures::pin_mut!(timeout, event);

            match iced::futures::future::select(event, timeout).await {
                iced::futures::future::Either::Left(_) => {
                    let _ = output.send(()).await;
                }
                iced::futures::future::Either::Right(_) => {}
            }
        }
    })
}
