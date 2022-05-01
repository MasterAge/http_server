use rtfmt::StringTemplate;

const HTML_TEMPLATE: &str = "
<!DOCTYPE HTML PUBLIC '-//W3C//DTD HTML 4.01//EN' 'http://www.w3.org/TR/html4/strict.dtd'>
<html>
<head>
<meta http-equiv='Content-Type' content='text/html; charset=utf-8'>
<title>Directory listing for ~{dir}</title>
</head>
<body>
<h1>Directory listing for ~{dir}</h1>
<hr>
~{file_list}
<hr>
</body>
</html>
";

pub fn file_list_to_html(files: Vec<String>, directory: String) -> String {
    let file_list_items:Vec<String> = files.iter()
        .map(|file| format!("<li><a href='{file}'>{file}</a></li>", file=file))
        .collect();

    let mut file_list = vec!["<ul>".to_string()];
    file_list.extend(file_list_items);
    file_list.push("</ul>".to_string());

    let mut template = StringTemplate::new(HTML_TEMPLATE);
    template.set("dir", directory);
    template.set("file_list", file_list.join("\n"));
    template.format()
}