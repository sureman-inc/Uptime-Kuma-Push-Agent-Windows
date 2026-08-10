fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico"); // Укажите путь к вашему файлу иконки
        res.compile().unwrap();
    }
}