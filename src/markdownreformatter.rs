use eframe::{epaint::{Color32, FontId, text::{LayoutJob, TextFormat}}};


pub struct Reformatter {
    cache: Vec<Line>,
    last_input: String,
    current_format: LayoutJob,
}

#[derive(Clone)]
pub struct Cache {
    pub text: String, 
    pub format: TextFormat,
}

#[derive(Clone)]
pub struct Line {
    pub text: String, 
    pub cached: bool, // if false run reformatter on this line, otherwise ignore
    pub cursor: bool, // is the cursor on this line
    pub sections: Vec<Cache>,
}


impl Reformatter {
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            last_input: String::new(),
            current_format: LayoutJob::default(),
        }
    }

    pub fn reformat(&mut self, text: &str, color: Color32, font: FontId) {
        self.last_input = text.to_string();
        self.current_format = LayoutJob::default();
        self.cache = self.break_to_lines(text);

        for line in &mut self.cache {
            if !line.cached {
                line.sections = Self::format_line(line, color, font.clone());
                line.cached = true
            }
        }

        self.commit_format();
    }

    /*
     * break_to_lines
     * Borrows mutable version of self
     * text: read only string
     * 
     * breaks text into lines, split by newline character
     * 
     * checks if each line is in the current cache, 
     * if a line is changed update it and mark it as uncached
     * otherwise mark the line as cached
     * 
     * if a line is new add a new uncached line to the vector
     * 
     * returns a vecotr of Struct Line
     */
    fn break_to_lines(&mut self, text: &str) -> Vec<Line> {
        let mut line_cache: Vec<Line> = Vec::new();

        for (current, line) in text.lines().enumerate() {
            if current < self.cache.len() {
                let old: Line = self.cache[current].clone();

                if old.text == line {
                    line_cache.push(old.clone());
                    line_cache[current].cached = true;
                } else {
                    line_cache.push(Line {
                        text: line.to_string(),
                        cached: false,
                        cursor: false,
                        sections: Vec::new(),
                    });
                }
            } else {
                line_cache.push(Line {
                    text: line.to_string(),
                    cached: false,
                    cursor: false,
                    sections: Vec::new(),

                });
            }
        }

        line_cache
    }
    
    /*
     * format_line
     * Borrows a Struct Line
     * color: color of text
     * default_font: mainly used for font family, also holds default font size
     * 
     * determines how a line should be formatted
     */
    fn format_line(line: &Line, color: Color32, default_font: FontId) -> Vec<Cache> {
        let mut font: FontId = FontId::default();
        let mut sections: Vec<Cache> = Vec::new();

        //check if we are a header, change font size if we are
        if let Some(level) = Self::header_level(&line.text) {
            font = Self::header_font(&default_font, level);
        }

        //Handle Italics
        if let Some(mut italics) = Self::format_italics(&line.text, color, font.clone()) {
            sections.append(&mut italics);
        }

        //if no other formats are found we are a plain string
        if sections.is_empty() {
            sections.push(Cache {
                text: line.text.clone(),
                format: Self::standard(color, font.clone()),
            });
        }

        sections
    }
    
    //handles inline italics formatting
    fn format_italics(text: &str, color: Color32, font: FontId) -> Option<Vec<Cache>> {
        let start = text.find('*')?; //is there a inline italic start
        let end = text[start + 1..].find('*')?; //does the inline italic end
        let end = start + 1 + end; //move end over one end * should also be italic

        let mut sections = Vec::new();

        //if we are in the middle of the line make sure the text prior to italics is standard
        if start > 0 {
            sections.push(Cache {
                text: text[..start].to_string(),
                format: Self::standard(color, font.clone()),
            });
        }

        //format inline italics
        sections.push(Cache {
            text: text[start..=end].to_string(),
            format: Self::italics(color, font.clone()),
        });

        //check the rest of the line
        if end + 1 < text.len() {
            if let Some(mut extra_sections) = Self::format_italics(&text[end + 1..], color, font.clone()) {
                sections.append(&mut extra_sections);
            } else {
                sections.push(Cache {
                    text: text[end + 1..].to_string(),
                    format: Self::standard(color, font.clone()),
                });
            }
        }

        Some(sections)
    }

    //determine header size
    fn header_level(text: &str) -> Option<usize> {
        let mut level = 0;

        for c in text.chars() {
            if c == '#' {
                level += 1;
            } else {
                break;
            }
        }

        if level >= 1 && level <= 4 {
            Some(level)
        } else {
            None
        }
    }

    //header sizes, supports up to h4
    fn header_font(default_font: &FontId, level: usize) -> FontId {
        let size = match level {
            1 => 32.0,
            2 => 24.0,
            3 => 18.72,
            4 => 16.0,
            _ => default_font.size,
        };

        FontId::new(size, default_font.family.clone())
    }
    
    //standard format getter
    fn standard(color: Color32, font: FontId) -> TextFormat {
        TextFormat {
            color: color,
            font_id: font,
            ..Default::default()
        }
    }
    
    //italic format getter
    fn italics(color: Color32, font: FontId) -> TextFormat {
        TextFormat {
            color: color,
            italics: true,
            font_id: font,
            ..Default::default()
        }
    }

    //join the formats of each line into one and set it as the current format
    fn commit_format(&mut self)
    {
        self.current_format = LayoutJob::default();

        for (i, line) in self.cache.iter().enumerate() {
            for section in &line.sections {
                self.current_format.append(
                    &section.text, 
                    0.0, 
                    section.format.clone(),
                );
            }

            if i + 1 < self.cache.len() {
                self.current_format.append(
                    "\n",
                    0.0,
                    TextFormat::default(),
                );
            }
        }
    }

    //check if input has changed
    pub fn needs_reformat(&self, input: &str) -> bool {
        self.last_input != input
    }

    //getter, returns current format
    pub fn formatted(&self) -> &LayoutJob {
        &self.current_format 
    }

}
