use std::collections::HashMap;

use eframe::{egui::Stroke, epaint::{Vec2, Color32, FontId, text::{LayoutJob, TextFormat}}};

pub static MAX_IMAGE_WIDTH: f32 = 600.0;

pub struct Reformatter {
    cache: Vec<Line>,
    last_input: String,
    current_format: LayoutJob,
    image_sizes: HashMap<String, Vec2>,
    last_color: Color32,
    last_font: FontId,
}

#[derive(Clone)]
pub struct Cache {
    pub text: String, 
    pub format: TextFormat,
    pub marker: bool,
    pub cursor: bool,
    pub group: usize,
    pub image: Option<String>
}

#[derive(Clone)]
pub struct Line {
    pub text: String, 
    pub cached: bool, // if false run reformatter on this line, otherwise ignore
    pub sections: Vec<Cache>,
}


impl Reformatter {

    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            last_input: String::new(),
            current_format: LayoutJob::default(),
            image_sizes: HashMap::new(),
            last_color: Color32::WHITE,
            last_font: FontId::default(),
        }
    }

    pub fn set_image_size(&mut self, path: &str, size: Vec2) {
        if self.image_sizes.get(path) != Some(&size) {
            self.image_sizes.insert(path.to_string(), size);
            self.commit_format(self.last_color, self.last_font.clone());
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

        self.commit_format(color, font);
    }

    pub fn set_cursor_pos(&mut self, pos: usize, color: Color32, font: FontId) {
        let mut current_range = 0;

        for line in &mut self.cache {
            let line_start = current_range;
            let line_end = line_start + line.text.chars().count();

            if pos >= line_start && pos <= line_end {
                let section_start = line_start;
                let mut cursor_group = 0;

                for section in &mut line.sections {
                    let section_end = section_start + section.text.chars().count();

                    if pos >= section_start && pos <= section_end {
                        cursor_group = section.group;
                        break;
                    }
                }
                for section in &mut line.sections {
                    section.cursor = cursor_group != 0 && section.group == cursor_group;
                }
            } else {
                for section in &mut line.sections {
                    section.cursor = false;
                }
            }

            current_range = line_end + 1;
        }

        self.commit_format(color, font);
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
                let mut old: Line = self.cache[current].clone();

                if old.text == line {
                    old.cached = true;
                    line_cache.push(old.clone());
                } else {
                    line_cache.push(Line {
                        text: line.to_string(),
                        cached: false,
                        sections: Vec::new(),
                    });
                }
            } else {
                line_cache.push(Line {
                    text: line.to_string(),
                    cached: false,
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
        let mut font: FontId = default_font.clone();
        let mut group = 0;

        //start with default formatting 
        let mut sections = vec![Cache {
            text: line.text.clone(),
            format: Self::standard(color, font.clone()),
            marker: false,
            cursor: false,
            group: 0,
            image: None,
        }];
        //check if we are a header, change font size if we are
        if let Some(level) = Self::header_level(&line.text) {
            font = Self::header_font(&default_font, level);
            sections = Self::format_header(sections, &mut group, color, font.clone());
        }


        //Handle Italics
        sections = Self::format_italics(sections, &mut group, color, font.clone());
        //Handle Strikethrough
        sections = Self::format_strikethrough(sections, &mut group, color, font.clone());
        //Handle Images
        sections = Self::format_images(sections, &mut group, color, font.clone());


        sections
    }
    
    //handles inline italics formatting
    fn format_italics(original: Vec<Cache>, group: &mut usize, color: Color32, font: FontId) -> Vec<Cache> {
        let mut sections = Vec::new();

        for section in original {
            let current_format = section.format;

            let Some(start) = section.text.find('*') else {
                sections.push(Cache {
                    text: section.text,
                    format: current_format,
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });

                continue;
            };

            let Some(relative_end) = section.text[start + 1..].find('*') else {
                sections.push(Cache {
                    text: section.text,
                    format: current_format,
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });

                continue;
            };

            let end = start + 1 + relative_end; 

    

            //if we are in the middle of the line make sure the text prior to italics is standard
            if start > 0 {
                sections.push(Cache {
                    text: section.text[..start].to_string(),
                    format: current_format.clone(),
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });
            }

            *group += 1;
            let current_group = *group;

            sections.push(Cache {
                text: section.text[start..=start].to_string(),
                format: Self::italics(current_format.clone(), color, font.clone()),
                marker: true,
                cursor: section.cursor,
                group: current_group,
                image: None,
            });

            //format inline italics
            sections.push(Cache {
                text: section.text[start + 1..end].to_string(),
                format: Self::italics(current_format.clone(), color, font.clone()),
                marker: false,
                cursor: section.cursor,
                group: current_group,
                image: None,
            });

            sections.push(Cache {
                text: section.text[end..=end].to_string(),
                format: Self::italics(current_format.clone(), color, font.clone()),
                marker: true,
                cursor: section.cursor,
                group: current_group,
                image: None,
            });



            //check the rest of the line
            if end + 1 < section.text.len() {
                let remaining = Cache {
                    text: section.text[end + 1..].to_string(),
                    format: current_format.clone(),
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                };
                let remaining = vec![remaining];

                let mut extra_sections = Self::format_italics(remaining, group, color, font.clone());
                sections.append(&mut extra_sections);
                
            }
        }
        
        sections
    }

    fn format_strikethrough(original: Vec<Cache>, group: &mut usize, color: Color32, font: FontId) -> Vec<Cache> {
        let mut sections = Vec::new();

        for section in original {
            let current_format = section.format;

            let Some(start) = section.text.find("~~") else {
                sections.push(Cache {
                    text: section.text,
                    format: current_format,
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });

                continue;
            };

            let Some(relative_end) = section.text[start + 2..].find("~~") else {
                sections.push(Cache {
                    text: section.text,
                    format: current_format,
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });

                continue;
            };

            let end = start + 2 + relative_end; 

    

            //if we are in the middle of the line make sure the text prior to italics is standard
            if start > 0 {
                sections.push(Cache {
                    text: section.text[..start].to_string(),
                    format: current_format.clone(),
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });
            }

            *group += 1;
            let current_group = *group;

            sections.push(Cache {
                text: section.text[start..=start + 1].to_string(),
                format: Self::strikethrough(current_format.clone(), color, font.clone()),
                marker: true,
                cursor: section.cursor,
                group: current_group,
                image: None,
            });
            
            //format inline italics
            sections.push(Cache {
                text: section.text[start + 2..end].to_string(),
                format: Self::strikethrough(current_format.clone(), color, font.clone()),
                marker: false,
                cursor: section.cursor,
                group: current_group,
                image: None,
            });
            
            sections.push(Cache {
                text: section.text[end..=end + 1].to_string(),
                format: Self::strikethrough(current_format.clone(), color, font.clone()),
                marker: true,
                cursor: section.cursor,
                group: current_group,
                image: None,
            });



            //check the rest of the line
            if end + 2 < section.text.len() {
                let remaining = Cache {
                    text: section.text[end + 2..].to_string(),
                    format: current_format.clone(),
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                };
                let remaining = vec![remaining];

                let mut extra_sections = Self::format_strikethrough(remaining, group, color, font.clone());
                sections.append(&mut extra_sections);
                
            }
        }
        
        sections
    }

    fn format_images(original: Vec<Cache>, group: &mut usize, color: Color32, font: FontId) -> Vec<Cache> {
        let mut sections = Vec::new();

        for section in original {
            let current_format = section.format;

            // find "!["
            let Some(bracket_start) = section.text.find("![") else {
                sections.push(Cache {
                    text: section.text,
                    format: current_format,
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });

                continue;
            };

            // find closing "]" after "!["
            let Some(relative_bracket_end) = section.text[bracket_start + 2..].find(']') else {
                sections.push(Cache {
                    text: section.text,
                    format: current_format,
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });

                continue;
            };

            let bracket_end = bracket_start + 2 + relative_bracket_end;

            //must be follwoed by (
            if section.text[bracket_end + 1..].chars().next() != Some('(') {
                sections.push(Cache {
                    text: section.text,
                    format: current_format,
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });

                continue;
            }
            let paren_start = bracket_end + 1;

            //find )
            let Some(relative_paren_end) = section.text[paren_start + 1..].find(')') else {
                sections.push(Cache {
                    text: section.text,
                    format: current_format,
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });

                continue;
            };
            let paren_end = paren_start + 1 + relative_paren_end;

            let path = section.text[paren_start + 1..paren_end].to_string();
            let raw_syntax = section.text[bracket_start..=paren_end].to_string();


            if bracket_start > 0 {
                    sections.push(Cache {
                    text: section.text[..bracket_start].to_string(),
                    format: current_format.clone(),
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                });
            }
            
            *group += 1;
            let current_group = *group;

            // raw markdown syntax, hidden unless cursor is on this group
            sections.push(Cache {
                text: raw_syntax[..1].to_string(),
                format: Self::standard(color, font.clone()),
                marker: true,
                cursor: section.cursor,
                group: current_group,
                image: Some(path),
            });

            sections.push(Cache {
                text: raw_syntax[1..].to_string(),
                format: Self::standard(color, font.clone()),
                marker: true,
                cursor: section.cursor,
                group: current_group,
                image: None,
            });

            //check the rest of the line
            if paren_end + 1 < section.text.len() {
                let remaining = Cache {
                    text: section.text[paren_end + 1..].to_string(),
                    format: current_format.clone(),
                    marker: section.marker,
                    cursor: section.cursor,
                    group: section.group,
                    image: None,
                };
                let remaining = vec![remaining];

                let mut extra_sections = Self::format_images(remaining, group, color, font.clone());
                sections.append(&mut extra_sections);
            }

        }

        sections
    }

    fn format_header(original: Vec<Cache>, group: &mut usize, color: Color32, font: FontId) -> Vec<Cache> {
        let mut sections = Vec::new();

        for section in original {
            *group += 1;
            let current_group = *group;

            let mut char_indices = section.text.char_indices();

            while let Some((i, c)) = char_indices.next() {
                if c == '#' {
                    sections.push(Cache {
                        text: c.to_string(),
                        format: Self::standard(color, font.clone()),
                        marker: true,
                        cursor: section.cursor,
                        group: current_group,
                        image: None,
                    });
                } else if c == ' ' {
                    sections.push(Cache {
                        text: c.to_string(),
                        format: Self::standard(color, font.clone()),
                        marker: true,
                        cursor: section.cursor,
                        group: current_group,
                        image: None,
                    });

                    let rest_start = i + c.len_utf8();
                    if rest_start < section.text.len() {
                        sections.push(Cache {
                            text: section.text[rest_start..].to_string(),
                            format: Self::standard(color, font.clone()),
                            marker: false,
                            cursor: section.cursor,
                            group: current_group,
                            image: None,
                        });
                    }
                    return sections;
                } else {
                    sections.push(Cache {
                        text: section.text[i..].to_string(),
                        format: Self::standard(color, font.clone()),
                        marker: false,
                        cursor: section.cursor,
                        group: current_group,
                        image: None,
                    });
                    return sections;
                }
            }
        }

        sections
    }

    //determine header size
    fn header_level(text: &str) -> Option<usize> {
        let mut level = 0;

        for c in text.chars() {
            if c == '#' {
                level += 1;
            } else  if c == ' ' {
                break;
            } else {
                level = 0;
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
    fn italics(old: TextFormat, color: Color32, font: FontId) -> TextFormat {
        TextFormat {
            color: color,
            italics: true,
            font_id: font,
            ..old
        }
    }

    //strikethrough getter
    fn strikethrough(old: TextFormat, color: Color32, font: FontId) -> TextFormat {
        TextFormat {
            color: color,
            strikethrough: Stroke::new(2.0_f32, color),
            font_id: font,
            ..old
        }
    }

    fn scaled_height(natural: Vec2, max_width: f32) -> f32 {
        let scale = (max_width / natural.x).min(1.0); // never upscale past natural size
        natural.y * scale
    }

    //join the formats of each line into one and set it as the current format
    fn commit_format(&mut self, color: Color32, font: FontId) {
        self.last_color = color;
        self.last_font = font.clone();
        self.current_format = LayoutJob::default();

        for (i, line) in self.cache.iter().enumerate() {
            for section in &line.sections {
                let format: TextFormat;
                if section.cursor {
                    format = Self::standard(color, FontId { 
                        size: f64::max(font.size.into(), 
                        section.format.font_id.size.into()) as f32, 
                        family: font.clone().family
                    });
                } else if let Some(path) = &section.image {
                    let height = self.image_sizes
                        .get(path)
                        .map(|size| Self::scaled_height(*size, MAX_IMAGE_WIDTH))
                        .unwrap_or(font.size * 6.0);


                    format = TextFormat {
                        color: Color32::TRANSPARENT,
                        font_id: FontId::new(height, section.format.font_id.family.clone()), // reserved height, tune later
                        ..Default::default()
                    }

                } else if section.marker {
                    format = TextFormat {
                        color: color,
                        font_id: FontId::new(0.0, section.format.font_id.family.clone()),
                        ..Default::default()
                    }
                } else {
                    format = section.format.clone();
                }

                self.current_format.append(
                    &section.text, 
                    0.0, 
                    format,
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

    /// Returns (line_index, path) for every line that contains an image.
    /// Assumes at most one image per line (block-level images only).
    pub fn image_lines(&self) -> Vec<(usize, String)> {
        self.cache
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                line.sections
                    .iter()
                    .find(|s| s.image.is_some() && !s.cursor)
                    .and_then(|s| s.image.clone())
                    .map(|path| (i, path))
            })
            .collect()
    }

}
