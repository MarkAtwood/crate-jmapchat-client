slint::slint! {
    export component AppWindow inherits Window {
        title: "JMAP Chat";
        preferred-width: 800px;
        preferred-height: 600px;

        VerticalLayout {
            alignment: center;
            spacing: 8px;
            padding: 16px;

            Text {
                text: "JMAP Chat";
                font-size: 24px;
                horizontal-alignment: center;
            }
            Text {
                text: "Coming soon";
                horizontal-alignment: center;
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    AppWindow::new()?.run()
}
