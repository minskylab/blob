use std::path::Path;

pub fn interpretation_prompt_template(file: &Path, file_source: String, prompt: String) -> String {
    format!(
        "
    # {}
    ```
    {}
    ```

    {}:

    ",
        file.display(),
        file_source,
        prompt
    )
}
