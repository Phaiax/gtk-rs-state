
# Global state for gtk-rs

[reddit question](https://www.reddit.com/r/rust/comments/9uz3qn/what_is_the_best_way_to_structure_a_gtk_rust/)

Runs on stable.

This is not a library and not on crates.io because I am not sure if this is
idiomatic and if the usage of unsafe is valid. If you want to use this,
just copy the `src/withgtk.rs` into your project.

This is not the fastest implementation, but for almost all
use cases this should be enough.

## Cargo.toml

You can use the newest versions, but you need glib next to gtk.

```toml
[dependencies]
lazy_static = "*"

[dependencies.gtk]
version = "0.5.0"
features = ["v3_10"]

[dependencies.glib]
version = "0.6.0"
```

## Example usage

Note that the function `external_element_access` needs no reference to the widgets.

```rust
gtk_refs!(
    widgets:                        // The macro emits a new module with this name
    gtk::Window => main_window,     // Widgettype => widget_name_from_glade
    gtk::Button => button1,
    gtk::Button => button2,
    gtk::Button => button3,
    gtk::Entry => entry1
);

fn main() {
    gtk::init().unwrap();
    let glade_src = include_str!("../ui.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    widgets::store_refs(&builder);

    // Optional: You can use the WidgetRefs type as a helper in
    // the main thread for yourself.
    let refs = widgets::WidgetRefs::from(&builder);

    // This type has a function for each of your widgets.
    // These functions return a clone() of the widget.
    refs.main_window().show_all();

    refs.main_window().connect_delete_event(move |_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Start event loop and some other thread
    std::thread::spawn(external_element_access);

    gtk::main();
 }

 fn external_element_access()  {
    let blub : &str = "Blub!";

    widgets::with_refs(|refs| {
        refs.entry1().set_text(blub); // Note the borrowing
    });
 }

```

## How does it work?

The closure you provide to `with_refs` is executed on the gtk event loop
via `glib::idle_add`. `with_refs` does wait until the closure has run.

Closures from multiple threads will always run sequentially.


# Notes:


## Unsafe

There is one usage of unsafe which is for convenience only.
You can easily remove the unsafe, but then you are forced to
`move` everything into the `with_ref` closure.


