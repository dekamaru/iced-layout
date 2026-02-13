use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::SystemTime;

use iced::widget;
use iced::{Element, Subscription};
use iced_layout_core::Node;

fn build<'a, Message: 'a>(node: &Node) -> Element<'a, Message> {
    match node {
        Node::Text(content) => widget::text(content.clone()).into(),
        Node::Container { id, children } => {
            assert_eq!(
                children.len(),
                1,
                "<container> must have exactly 1 child element, found {}",
                children.len()
            );
            let child = build::<Message>(&children[0]);
            let c = widget::container(child);
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
