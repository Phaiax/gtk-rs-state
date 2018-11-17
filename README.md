
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


```rust
    gtk_refs!(
        pub mod widgets;                // The macro emits a new module with this name
        struct WidgetRefs;              // The macro emits a struct with this name containing:
        main_window : gtk::Window ,     // widget_name : Widgettype
        button1 : gtk::Button           // ..
    );

    fn main() {

        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        let window = Window::new(WindowType::Toplevel);
        window.set_title("gtk-rs-state Example Program");
        window.set_default_size(350, 70);
        let button = Button::new_with_label("Spawn another thread!");
        window.add(&button);
        window.show_all();

        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        button.connect_clicked(|_| {
            std::thread::spawn(some_workfunction);
            println!("Clicked!");
        });

        // You need the following two statements to prepare the
        // static storage needed for cross thread access.
        // See the `from_glade.rs` example for a more elegant solution
        let widget_references = widgets::WidgetRefs {
            main_window: window.clone(),
            button1:     button.clone(),
        };

        widgets::init_storage(widget_references);
        // End

        // This type has a function for each of your widgets.
        // These functions return a clone() of the widget.
        window.show_all();

        window.connect_delete_event(move |_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        // Start event loop
        gtk::main();
    }

    fn compute() {
        use std::thread::sleep;
        use std::time::Duration;
        sleep(Duration::from_secs(1));
    }

    fn some_workfunction()  {
        let mut i = 0;

        loop {
            compute();

            i += 1;
            let text = format!("Round {} in {:?}", i, std::thread::current().id());

            widgets::do_in_gtk_eventloop(|refs| {
                refs.button1().set_label(&text);
            });
        }
    }
```


The macro generates the following code:

```rust
    pub mod widgets {
        pub struct WidgetRefs {
            pub main_window : gtk::Window,
            ...
        }
        impl From<&gtk::Builder> for WidgetRefs { ... };
        impl WidgetRefs {
            fn main_window() -> gtk::Window { } // returns a .clone() of the widget
            ...
        }

        pub fn init_storage(WidgetRefs);
        pub fn init_storage_from_builder(&gtk::Builder);
        pub fn do_in_gtk_eventloop( FnOnce(Rc<WidgetRefs>) );
    }
```


## How does it work?

 - The closure you provide to `do_in_gtk_eventloop(closure)` is executed on the gtk event loop via `glib::idle_add()`.
 - `do_in_gtk_eventloop()` does wait until the closure has run.
 - Closures from multiple threads will always run sequentially.
 - If the closure panics, `do_in_gtk_eventloop()` will panic as well. You may not see the panic because the process will exit too fast.

## Please also see the examples folder if you n.


## Use of `unsafe`

There is one usage of unsafe which is for convenience only. It allows the closure to reference the local stack instead of requiring 'static on the closure.

You can easily remove the unsafe, but then you are forced to `move` everything into the `with_ref` closure.


