use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

pub struct TNT {
    filename: String,
}

impl TNT {
    /// Создаёт новый экземпляр TNT, ассоциированный с указанным файлом.
    /// Если файл не существует, он будет создан.
    ///
    /// # Аргументы
    /// * `filename` - Имя файла для хранения данных.
    ///
    /// # Пример
    /// ```
    /// let tnt = TNT::connect("data.txt");
    /// ```
    ///
    /// Creates a new TNT instance associated with the specified file.
    /// If the file does not exist, it will be created.
    ///
    /// # Arguments
    /// * `filename` - The name of the file to store data.
    ///
    /// # Example
    /// ```
    /// let tnt = TNT::connect("data.txt");
    /// ```
    pub fn connect(filename: &str) -> Self {

        if fs::metadata(filename).is_err() {
            File::create(filename).expect("File creation error!");
        }

        Self { filename: filename.to_string()}
    }

    fn is_ccf(&self) -> std::io::Result<bool> {
        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut is_good = false;
        let mut brack = 0;
        let mut rev_brack = 0;
        let mut braces = 0;
        let mut rev_braces = 0;

        for line in reader.lines() {
            let line = line?;

            if line.find('(').is_some() {
                brack += 1;
            }

            if line.find(')').is_some() {
                rev_brack += 1;
            }

            if line.find('{').is_some() {
                braces += 1;
            }

            if line.find('}').is_some() {
                rev_braces += 1;
            }
        }

        if brack == rev_brack && braces == rev_braces {
            is_good = true;
        }

        Ok(is_good)
    }

    fn get_var_line(&self, key: &str, var: &str) -> std::io::Result<i32> {
        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut is_key = false;
        let mut var_line: i32 = -1;

        for (i, line) in reader.lines().enumerate() {
            let line = line?;

            if line.find('(').is_some() {
                let cleaned_line = line.replace(['(', ')', '{'], "");
                let k = cleaned_line.trim();

                if k == key {
                    is_key = true;
                    continue;
                }
            }

            if is_key && line.find('}').is_some() {
                break;
            }


            if is_key {
                let fmt_line = line.trim();
                let var_from_line = fmt_line.split('=').next().unwrap_or(fmt_line);

                if var_from_line == var.trim() {
                    var_line = i as i32;
                    break;

                }
            }

        }

        Ok(var_line)
    }

    fn is_var(&self, key: &str, var: &str) -> std::io::Result<bool> {
        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut is_varible = false;
        let mut is_found_key = false;
        let mut var_val = String::new();

        for line in reader.lines() {
            let line = line?;

            if line.find('(').is_some() {
                let cleaned_line = line.replace(['(', ')', '{'], "");
                let k = cleaned_line.trim();

                if k == key.trim() {
                    is_found_key = true;
                    continue;
                }
            }

            if is_found_key && line.find('}').is_some() {
                break;
            }

            if is_found_key {
                let fmt_line = line.trim();
                var_val.push_str(&format!("{},", fmt_line));
            }
        }

        let old_var: Vec<&str > = var_val.split(',').filter(|s| !s.is_empty()).collect();
        
        for v in old_var {
            let v_v = v.split('=').next().unwrap_or(v);

            if v_v == var.trim() {
                is_varible = true;
                break;
            }
        }

        Ok(is_varible)

    }

    /// Добавляет новую переменную с указанным значением в секцию ключа.
    /// Если ключа нет, он будет создан. Если переменная уже существует, операция не выполнится.
    ///
    /// # Аргументы
    /// * `key` - Имя секции (ключа)
    /// * `var` - Имя переменной
    /// * `val` - Значение переменной
    ///
    /// Adds a new variable with the specified value to the key section.
    /// If the key does not exist, it will be created. If the variable already exists, the operation will not be performed.
    ///
    /// # Arguments
    /// * `key` - Section (key) name
    /// * `var` - Variable name
    /// * `val` - Variable value
    pub fn add<T: std::fmt::Display, V: std::fmt::Display>(&self, key: &str, var: T, val: V) -> std::io::Result<()> {
        let var_str = var.to_string();
        let val_str = val.to_string();

        if !self.is_ccf()? {
            println!("File integrity error!");
            return Ok(());
        }
        
        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut is_found = false;
        let mut is_not_key = true;
        let mut new_text: String = String::new();

        for line in reader.lines() {
            let line = line?;
            
            if line.find('(').is_some() {
                let cleaned_line = line.replace('(', "").replace(')', "").replace('{', "");
                let k = cleaned_line.trim();
                
                if k == key {
                    is_found = true;
                    is_not_key = false;
                }
            }

            if line.find('}').is_some() && is_found {
                let is_val = self.is_var(key, var_str.as_str())?;

                if is_val {
                    println!("A variable named '{}' already exists!", var_str);
                    return Ok(());
                } 

                new_text.push_str(&format!("\t{}={}\n", var_str, val_str));
                is_found = false;
            }

            new_text.push_str(&line);
            new_text.push_str("\n");
            
        }

        if is_not_key {
            new_text.push_str(&format!("({}) {{\n\t{}={}\n}}", key, var_str, val_str));
        }
       

        let mut file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&self.filename)?;
        write!(file, "{}", new_text)?;

        Ok(())
    }

    /// Получает значение переменной по ключу и имени переменной.
    /// Если переменная не найдена, возвращает "NONE_VAL".
    ///
    /// # Аргументы
    /// * `key` - Имя секции (ключа)
    /// * `var` - Имя переменной
    ///
    /// Gets the value of a variable by key and variable name.
    /// If the variable is not found, returns "NONE_VAL".
    ///
    /// # Arguments
    /// * `key` - Section (key) name
    /// * `var` - Variable name
    pub fn get<T: std::fmt::Display>(&self, key: &str, var: T) -> std::io::Result<String> {
        let var_str = var.to_string();
        
        if !self.is_ccf()? {
            println!("File integrity error!");
            return Ok("Error".to_string());
        }

        let file = fs::File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut is_key = false;
        let mut fmt_val = String::new();


        for line in reader.lines() {
            let line = line?;

            if line.find('(').is_some() {
                let cleaned_line = line.replace(['(', ')', '{'], "");
                let k = cleaned_line.trim();
                
                if k == key {
                    is_key = true;
                    continue;
                }
            }

            if is_key && line.find('}').is_some() {
                println!("The variable was not found!");
                break;
            }


            if is_key {
                if let Some((name, val)) = line.trim().split_once("=") {
                    if name == var_str.trim() {
                        fmt_val.push_str(val);
                        break;
                    }
                }
            }

        }


        if fmt_val.is_empty() {
            fmt_val.push_str("NONE_VAL");
        }

        Ok(fmt_val)

    }

    /// Изменяет значение переменной в секции ключа.
    /// Если переменная не найдена, операция не выполнится.
    ///
    /// # Аргументы
    /// * `key` - Имя секции (ключа)
    /// * `var` - Имя переменной
    /// * `new_val` - Новое значение переменной
    ///
    /// Edits the value of a variable in the key section.
    /// If the variable is not found, the operation will not be performed.
    ///
    /// # Arguments
    /// * `key` - Section (key) name
    /// * `var` - Variable name
    /// * `new_val` - New value for the variable
    pub fn edit<T: std::fmt::Display, V: std::fmt::Display>(&self, key: &str, var: T, new_val: V) -> std::io::Result<()> {
        let var_str = var.to_string();
        let val_str = new_val.to_string();

        if !self.is_ccf()? {
            println!("File integrity error!");
            return Ok(());
        }

        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let pos = self.get_var_line(key, var_str.as_str())?;
        let mut txt = String::new();

        if pos == -1 {
            println!("The variable was not found!");
            return Ok(());
        }

        for (i, line) in reader.lines().enumerate() {
            let line = line?;

            if i as i32 == pos {
                txt.push_str(&format!("\t{}={}\n", var_str, val_str));
                continue;
            }

            txt.push_str(&line);
            txt.push_str("\n");
        }

        let mut file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&self.filename)?;
        write!(file, "{}", txt)?;

        Ok(())
    }

    /// Удаляет переменную из секции ключа.
    /// Если переменная не найдена, операция не выполнится.
    ///
    /// # Аргументы
    /// * `key` - Имя секции (ключа)
    /// * `var` - Имя переменной
    ///
    /// Deletes a variable from the key section.
    /// If the variable is not found, the operation will not be performed.
    ///
    /// # Arguments
    /// * `key` - Section (key) name
    /// * `var` - Variable name
    pub fn delete_var<T: std::fmt::Display>(&self, key: &str, var: T) -> std::io::Result<()> {
        let var_str = var.to_string();

        if !self.is_ccf()? {
            println!("File integrity error!");
            return Ok(());
        }

        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut txt = String::new();
        let var_del_line = self.get_var_line(key, var_str.as_str())?;

        if var_del_line == -1 {
            println!("The variable was not found!");
            return Ok(());
        }

        for (i, line) in reader.lines().enumerate() {
            let line = line?;

            if i == var_del_line as usize {
                continue;
            }

            txt.push_str(&line);
            txt.push_str("\n");
        }

        let mut file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&self.filename)?;
        write!(file, "{}", txt)?;

        Ok(())            
    }

    /// Удаляет секцию (ключ) и все переменные внутри неё.
    /// Если ключ не найден, операция не выполнится.
    ///
    /// # Аргументы
    /// * `key` - Имя секции (ключа)
    ///
    /// Deletes a section (key) and all variables inside it.
    /// If the key is not found, the operation will not be performed.
    ///
    /// # Arguments
    /// * `key` - Section (key) name
    pub fn delete_key(&self, key: &str) -> std::io::Result<()> {

        if !self.is_ccf()? {
            println!("File integrity error!");
            return Ok(());
        }

        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut txt = String::new();
        let mut is_found_key = false;
        let mut is_key = false;

        for line in reader.lines() {
            let line = line?;

            if line.find('(').is_some() {
                let cleaned_line = line.replace(['(', ')', '{'], "");
                let k = cleaned_line.trim();

                if k == key.trim() {
                    is_key = true;
                    is_found_key = true;
                    continue;
                }
            }

            if is_key && line.find('}').is_some() {
                is_key = false;
                continue;
            }

            if is_key {
                continue;
            }

            txt.push_str(&line);
            txt.push_str("\n");
        }

        if is_found_key {
            let mut file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(&self.filename)?;
            write!(file, "{}", txt)?;
        }

        Ok(())
    }

    /// Очищает весь файл, удаляя все данные.
    ///
    /// Clears the entire file, removing all data.
    pub fn clear(&self) -> std::io::Result<()> {
        File::create(&self.filename)?;
        Ok(())
    }

    /// Получает все значения переменных в секции ключа.
    ///
    /// # Аргументы
    /// * `key` - Имя секции (ключа)
    ///
    /// Gets all variable values in the key section.
    ///
    /// # Arguments
    /// * `key` - Section (key) name
    pub fn get_all(&self, key: &str) -> std::io::Result<Vec<String>> {

        if !self.is_ccf()? {
            println!("File integrity error!");
            return Ok(vec!["".to_string()]);
        }

        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut all_val: Vec<String> = Vec::new();
        let mut is_key = false;

        for line in reader.lines() {
            let line = line?;

            if line.find('(').is_some() {
                let cleaned_line = line.replace(['(', ')', '{'], "");
                let k = cleaned_line.trim();

                if k == key.trim() {
                    is_key = true;
                    continue;
                }
            }

            if is_key && line.find('}').is_some() {
                break;
            }

            if is_key {
                let fmt_line = line.split('=').nth(1).unwrap_or("");
                let val_fmt = fmt_line.trim().to_string();
                all_val.push(val_fmt);
            }
        }

        Ok(all_val)

    }

    /// Экспортирует данные в формате TOML в указанный файл.
    ///
    /// # Аргументы
    /// * `filename` - Имя файла (без расширения), куда будет сохранён TOML.
    ///
    /// Exports data in TOML format to the specified file.
    ///
    /// # Arguments
    /// * `filename` - File name (without extension) where TOML will be saved.
    pub fn to_toml(&self, filename: &str) -> std::io::Result<()> {

        if !self.is_ccf()? {
            println!("File integrity error!");
            return Ok(());
        }

        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut is_key = false;
        let mut txt: String = String::new();

        for line in reader.lines() {
            let line = line?;

            if line.find('(').is_some() {
                let cleaned_line = line.replace(['(', ')', '{'], "");
                let key = cleaned_line.trim();
                txt.push_str(&format!("[{}]\n", key));
                is_key = true;
                continue;
            }

            if is_key && line.find('}').is_some() {
                is_key = false;
                txt.push_str("\n");
                continue;
            }

            if is_key {
                let fmt_val = line.trim();
                let var = fmt_val.split('=').next().unwrap_or(fmt_val);
                let val = fmt_val.split('=').nth(1).unwrap_or("");
                txt.push_str(&format!("{}=\"{}\"\n", var, val));
            }

        }

        let full_filename = String::from(&format!("{}.toml", filename));
        let mut file = fs::OpenOptions::new().write(true).truncate(true).create(true).open(full_filename)?;
        write!(file, "{}", txt)?;
        
        Ok(())
    }

    /// Экспортирует данные в формате JSON в указанный файл.
    ///
    /// # Аргументы
    /// * `filename` - Имя файла (без расширения), куда будет сохранён JSON.
    ///
    /// Exports data in JSON format to the specified file.
    ///
    /// # Arguments
    /// * `filename` - File name (without extension) where JSON will be saved.
    pub fn to_json(&self, filename: &str) -> std::io::Result<()> {

        if !self.is_ccf()? {
            println!("File integrity error!");
            return Ok(());
        }

        let file = File::open(&self.filename)?;
        let reader = BufReader::new(&file);
        let mut txt: String = String::new();
        let mut is_key: bool = false;
        let mut val_str = String::new();

        txt.push_str("{\n");

        for line in reader.lines() {
            let line = line?;

            if line.find('(').is_some() {
                let cleaned_line = line.replace(['(', ')', '{'], "");
                let key = cleaned_line.trim();
                txt.push_str(&format!("\t\"{}\": {{\n", key));
                is_key = true;
                continue;
            }

            if is_key && line.find('}').is_some() {
                is_key = false;
                let mut new_len = val_str.len() - 2;

                while !val_str.is_char_boundary(new_len) && new_len > 0 {
                    new_len -= 1;
                }

                val_str.truncate(new_len);

                txt.push_str(&val_str);
                txt.push_str("\n\t},\n\n");
                val_str.clear();
                continue;
            }

            if is_key {
                let fmt_line = line.trim();
                let var = fmt_line.split('=').next().unwrap_or(fmt_line);
                let val = fmt_line.split('=').nth(1).unwrap_or("");
                val_str.push_str(&format!("\t\t\"{}\": \"{}\",\n", val, var));
            }

        }

        let mut new_len = txt.len() - 3;

        while !txt.is_char_boundary(new_len) && new_len > 0 {
            new_len -= 1;
        }

        txt.truncate(new_len);
        txt.push_str("\n}");

        let full_filename = String::from(&format!("{}.json", filename));
        let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(full_filename)?;
        writeln!(file, "{}", txt)?;

        Ok(())
    }

    /// Импортирует данные из TOML-файла в основной файл.
    ///
    /// # Аргументы
    /// * `toml` - Имя TOML-файла (без расширения), из которого будут импортированы данные.
    ///
    /// Imports data from a TOML file into the main file.
    ///
    /// # Arguments
    /// * `toml` - TOML file name (without extension) from which data will be imported.
    pub fn from_toml(&self, toml: &str) -> std::io::Result<()> {
        let full_toml = String::from(&format!("{}.toml", toml));
        let file = File::open(full_toml)?;
        let reader = BufReader::new(&file);
        let mut txt = String::new();
        let mut is_one = 0;

        for line in reader.lines() {
            let line = line?;

            if line.find('[').is_some() && line.find(']').is_some() {
                let cleaned_line = line.replace(['[', ']'], "");
                let key = cleaned_line.trim();
                if is_one > 0 {
                    txt.push_str(&format!("}}\n({}) {{\n", key));
                } else {
                    txt.push_str(&format!("({}) {{\n", key));
                }
                is_one += 1;
                continue;
            }

            if !line.is_empty() {
                let fmt_line = line.replace(['\"', ' '], "");
                let fmt_line_2 = fmt_line.trim();
                let var = fmt_line_2.split('=').next().unwrap_or(fmt_line_2);
                let val = fmt_line_2.split('=').nth(1).unwrap_or("");

                txt.push_str(&format!("\t{}={}\n", var, val));
            }
        }

        txt.push_str("}");

        let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(&self.filename)?;
        writeln!(file, "{}", txt)?;

        Ok(())
    }

    /// Импортирует данные из JSON-файла в основной файл.
    ///
    /// # Аргументы
    /// * `json` - Имя JSON-файла (без расширения), из которого будут импортированы данные.
    ///
    /// Imports data from a JSON file into the main file.
    ///
    /// # Arguments
    /// * `json` - JSON file name (without extension) from which data will be imported.
    pub fn from_json(&self, json: &str) -> std::io::Result<()> {
        let full_json = String::from(&format!("{}.json", json));
        let file = File::open(full_json)?;
        let reader = BufReader::new(&file);
        let mut txt = String::new();
        let mut is_key = false;


        for line in reader.lines() {
            let line = line?;

            if line.find('\"').is_some() && line.find("{").is_some() && line.find(':').is_some() {
                let cleaned_line = line.replace(['{', ':', '\"'], "");
                let key = cleaned_line.trim();
                txt.push_str(&format!("({}) {{\n", key));
                is_key = true;
                continue;
            }

            if is_key && line.find('}').is_some() {
                is_key = false;
                txt.push_str("}\n");
                continue;
            }

            if is_key && !line.is_empty() {
                let cleaned_line = line.replace(['\"', ',', ' '], "");
                let var_and_val = cleaned_line.trim();
                let var = var_and_val.split(':').next().unwrap_or(var_and_val);
                let val = var_and_val.split(':').nth(1).unwrap_or("");
                txt.push_str(&format!("\t{}={}\n", var, val));
            }
        }

        let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(&self.filename)?;
        writeln!(file, "{}", txt)?;

        Ok(())
    }
}
