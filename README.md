
# Modify gtk widgets from other threads

[reddit question](https://www.reddit.com/r/rust/comments/9uz3qn/what_is_the_best_way_to_structure_a_gtk_rust/)

Runs on stable.

This is not the fastest implementation, but for almost all
use cases this should be enough.

## Cargo.toml

You can use the newest versions, but you need glib next to gtk.

```toml
[dependencies]
gtk-fnonce-on-eventloop = "0.2"
```

This crate uses these dependencies:
 - gtk in version = "0.5.0" with features = ["v3_10"]
 - glib in version = "0.6.0"


## Example usage

Using this boils down to
 - Call the gtk_refs! macro.
 - Call init_storage() before starting the event loop
 - Call do_in_gtk_eventloop() in other threads to modify the widgets

Don't call do_in_gtk_eventloop() in the main thread since this will block.

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
        window.set_title("gtk-fnonce-on-eventloop Example Program");
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
 - At the callsite `do_in_gtk_eventloop()` does wait until the closure has run.
 - Closures from multiple threads will always run sequentially.
 - If the closure panics, `do_in_gtk_eventloop()` will panic as well. You may not see the panic because the process usually exits too fast.

Please also see the examples folder if you want to:
    - Use additional non-send fields in the struct (other stuff than widget references)
    - Use glade

## How does the implementation work?

```text
+---------------------------------+   +----------------------------------+
|GTK event loop thread            |   |Global statics                    |
|                                 |   |                                  |
|  +----------------------------+ |   |  TX : Sender<(Fn, Cb)>           |
|  |Thread local statics        | |   |                                  |
|  |                            | |   |                                  |
|  |  DATA : Non-Send Refereces | |   +----------------------------------+
|  |  RX : Receiver<(Fn, Cb)>   | |
|  |                            | |   +----------------------------------+
|  +----------------------------+ |   |Some other thread                 |
|                                 |   |                                  |
|                                 |   |  do some stuff                   |
|  +---------------------------+  |   |                                  |
|  |event loop()               |  |   |  call do_in_gtk_eventloop(Fn)    |
|  |                           |  |   |    This Fn has access to DATA    |
|  |  +----------------------+ |  |   |                                  |
|  |  |closure added with    | |  |   |                                  |
|  |  |idle_add() to execute | |  |   +----------------------------------+
|  |  |on the gtk thread {   | |  |
|  |  |                      <-----------------------------------------------+
|  |  |  Pop (Fn,Cb) from RX | |  |                                          |
|  |  |  Call Fn(DATA)       | |  |                                          |
|  |  |  Signal end of Fn    | |  |   +----------------------------------+   |
|  |  |   via Cb             | |  |   |do_in_gtk_eventloop(Fn)           |   |
|  |  |                      | |  |   |                                  |   |
|  |  |                      | |  |   |  Box closure Fn and transmute    |   |
|  |  +----------------------+ |  |   |   livetime to 'static            |   |
|  |                           |  |   |                                  |   |
|  +---------------------------+  |   |  Create a signal Cb              |   |
|                                 |   |  Push (boxed Fn, Cb) to TX       |   |
+---------------------------------+   |  Add this closure via idle_add() +---+
                                      |  Wait for the signal Cb          |
                                      |  return                          |
                                      |                                  |
                                      |                                  |
                                      +----------------------------------+
```

`init_storage()` initializes DATA, RX and TX.

## Use of `unsafe`

There is one usage of unsafe which is for convenience only. It allows the closure to reference the local stack instead of requiring 'static on the closure.

You can easily remove the unsafe, but then you are forced to `move` everything into the `with_ref` closure.


