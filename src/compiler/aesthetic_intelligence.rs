use super::constraint_engine::*;
use super::semantic_enhanced::*;
use super::capella_metamodel::*;
use super::safety_intelligence::SafetyLevel;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct AestheticIntelligence {
    color_schemes: HashMap<String, ColorScheme>,
    typography_rules: TypographyRules,
    spacing_rules: SpacingRules,
    visual_hierarchy: VisualHierarchy,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub name: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
    pub border: String,
    pub gradient_start: String,
    pub gradient_end: String,
}

#[derive(Debug, Clone)]
pub struct TypographyRules {
    pub title_font: String,
    pub title_size: f64,
    pub label_font: String,
    pub label_size: f64,
    pub annotation_font: String,
    pub annotation_size: f64,
    pub line_height: f64,
    pub letter_spacing: f64,
}

#[derive(Debug, Clone)]
pub struct SpacingRules {
    pub grid_size: f64,
    pub min_margin: f64,
    pub padding: f64,
    pub element_gap: f64,
    pub layer_spacing: f64,
    pub golden_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct VisualHierarchy {
    pub importance_weights: HashMap<String, f64>,
    pub size_scale: Vec<f64>,
    pub emphasis_elements: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct PolishSettings {
    pub anti_aliasing: bool,
    pub smooth_curves: bool,
    pub shadow_enabled: bool,
    pub shadow_blur: f64,
    pub shadow_offset_x: f64,
    pub shadow_offset_y: f64,
    pub shadow_color: String,
    pub corner_radius: f64,
    pub dpi: u32,
}

#[derive(Debug, Clone)]
pub struct GestaltPrinciples {
    pub proximity_threshold: f64,
    pub similarity_grouping: bool,
    pub continuity_enabled: bool,
    pub closure_enabled: bool,
}

impl Default for AestheticIntelligence {
    fn default() -> Self {
        let mut color_schemes = HashMap::new();
        
        color_schemes.insert("professional".to_string(), ColorScheme {
            name: "Professional Blue".to_string(),
            primary: "#2C3E50".to_string(),
            secondary: "#34495E".to_string(),
            accent: "#3498DB".to_string(),
            background: "#ECF0F1".to_string(),
            text: "#2C3E50".to_string(),
            border: "#BDC3C7".to_string(),
            gradient_start: "#3498DB".to_string(),
            gradient_end: "#2980B9".to_string(),
        });
        
        color_schemes.insert("elegant".to_string(), ColorScheme {
            name: "Elegant Gray".to_string(),
            primary: "#37474F".to_string(),
            secondary: "#546E7A".to_string(),
            accent: "#00897B".to_string(),
            background: "#FAFAFA".to_string(),
            text: "#263238".to_string(),
            border: "#B0BEC5".to_string(),
            gradient_start: "#00897B".to_string(),
            gradient_end: "#00695C".to_string(),
        });
        
        color_schemes.insert("modern".to_string(), ColorScheme {
            name: "Modern Teal".to_string(),
            primary: "#006064".to_string(),
            secondary: "#00838F".to_string(),
            accent: "#00ACC1".to_string(),
            background: "#E0F7FA".to_string(),
            text: "#004D40".to_string(),
            border: "#80DEEA".to_string(),
            gradient_start: "#00ACC1".to_string(),
            gradient_end: "#0097A7".to_string(),
        });
        
        color_schemes.insert("vibrant".to_string(), ColorScheme {
            name: "Vibrant Purple".to_string(),
            primary: "#4A148C".to_string(),
            secondary: "#6A1B9A".to_string(),
            accent: "#AB47BC".to_string(),
            background: "#F3E5F5".to_string(),
            text: "#311B92".to_string(),
            border: "#CE93D8".to_string(),
            gradient_start: "#AB47BC".to_string(),
            gradient_end: "#8E24AA".to_string(),
        });
        
        Self {
            color_schemes,
            typography_rules: TypographyRules::default(),
            spacing_rules: SpacingRules::default(),
            visual_hierarchy: VisualHierarchy::default(),
        }
    }
}

impl Default for TypographyRules {
    fn default() -> Self {
        Self {
            title_font: "Helvetica Neue, Arial, sans-serif".to_string(),
            title_size: 16.0,
            label_font: "Helvetica Neue, Arial, sans-serif".to_string(),
            label_size: 12.0,
            annotation_font: "Helvetica Neue, Arial, sans-serif".to_string(),
            annotation_size: 10.0,
            line_height: 1.5,
            letter_spacing: 0.02,
        }
    }
}

impl Default for SpacingRules {
    fn default() -> Self {
        Self {
            grid_size: 8.0,
            min_margin: 20.0,
            padding: 12.0,
            element_gap: 16.0,
            layer_spacing: 80.0,
            golden_ratio: 1.618,
        }
    }
}

impl Default for VisualHierarchy {
    fn default() -> Self {
        let mut importance_weights = HashMap::new();
        importance_weights.insert("system".to_string(), 1.0);
        importance_weights.insert("component".to_string(), 0.8);
        importance_weights.insert("function".to_string(), 0.7);
        importance_weights.insert("interface".to_string(), 0.6);
        importance_weights.insert("port".to_string(), 0.4);
        
        Self {
            importance_weights,
            size_scale: vec![1.0, 0.875, 0.75, 0.625, 0.5],
            emphasis_elements: HashSet::new(),
        }
    }
}

impl Default for PolishSettings {
    fn default() -> Self {
        Self {
            anti_aliasing: true,
            smooth_curves: true,
            shadow_enabled: true,
            shadow_blur: 4.0,
            shadow_offset_x: 2.0,
            shadow_offset_y: 2.0,
            shadow_color: "rgba(0, 0, 0, 0.15)".to_string(),
            corner_radius: 6.0,
            dpi: 300,
        }
    }
}

impl Default for GestaltPrinciples {
    fn default() -> Self {
        Self {
            proximity_threshold: 50.0,
            similarity_grouping: true,
            continuity_enabled: true,
            closure_enabled: true,
        }
    }
}

impl AestheticIntelligence {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn apply_visual_hierarchy(&self, elements: &mut Vec<DiagramElement>) {
        for element in elements.iter_mut() {
            let importance = self.visual_hierarchy.importance_weights
                .get(&element.element_type)
                .copied()
                .unwrap_or(0.5);
            
            element.size_multiplier = importance;
            element.visual_weight = importance;
            
            if importance > 0.7 {
                element.is_focal_point = true;
            }
        }
    }
    
    pub fn apply_gestalt_principles(&self, elements: &mut Vec<DiagramElement>, gestalt: &GestaltPrinciples) {
        if gestalt.similarity_grouping {
            self.group_by_similarity(elements);
        }
        
        if gestalt.continuity_enabled {
            self.apply_continuity(elements);
        }
        
        self.apply_proximity_grouping(elements, gestalt.proximity_threshold);
    }
    
    fn group_by_similarity(&self, elements: &mut Vec<DiagramElement>) {
        let mut type_groups: HashMap<String, Vec<usize>> = HashMap::new();
        
        for (idx, element) in elements.iter().enumerate() {
            type_groups.entry(element.element_type.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }
        
        for (element_type, indices) in type_groups {
            if indices.len() > 1 {
                for &idx in &indices {
                    elements[idx].group_id = Some(element_type.clone());
                }
            }
        }
    }
    
    fn apply_continuity(&self, elements: &mut Vec<DiagramElement>) {
        for element in elements.iter_mut() {
            element.enable_smooth_edges = true;
            element.edge_interpolation = EdgeInterpolation::Bezier;
        }
    }
    
    fn apply_proximity_grouping(&self, elements: &mut Vec<DiagramElement>, threshold: f64) {
        for i in 0..elements.len() {
            for j in (i + 1)..elements.len() {
                let distance = self.calculate_distance(&elements[i], &elements[j]);
                if distance < threshold {
                    elements[i].proximity_group = Some(format!("cluster_{}", i / 3));
                    elements[j].proximity_group = Some(format!("cluster_{}", i / 3));
                }
            }
        }
    }
    
    fn calculate_distance(&self, e1: &DiagramElement, e2: &DiagramElement) -> f64 {
        let dx = e1.x - e2.x;
        let dy = e1.y - e2.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    pub fn apply_balance(&self, elements: &mut Vec<DiagramElement>) {
        let total_weight: f64 = elements.iter().map(|e| e.visual_weight).sum();
        let target_weight_per_quadrant = total_weight / 4.0;
        
        let mut quadrant_weights = [0.0; 4];
        
        for element in elements.iter() {
            let quadrant = self.get_quadrant(element);
            quadrant_weights[quadrant] += element.visual_weight;
        }
        
        for (quadrant, &weight) in quadrant_weights.iter().enumerate() {
            if weight < target_weight_per_quadrant * 0.7 {
                self.redistribute_to_quadrant(elements, quadrant);
            }
        }
    }
    
    fn get_quadrant(&self, element: &DiagramElement) -> usize {
        let x_mid = element.x > 0.0;
        let y_mid = element.y > 0.0;
        
        match (x_mid, y_mid) {
            (false, false) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (true, true) => 3,
        }
    }
    
    fn redistribute_to_quadrant(&self, elements: &mut Vec<DiagramElement>, _quadrant: usize) {
        // Placeholder for redistribution logic
    }
    
    pub fn apply_rhythm(&self, elements: &mut Vec<DiagramElement>) {
        let grid_size = self.spacing_rules.grid_size;
        
        for element in elements.iter_mut() {
            element.x = (element.x / grid_size).round() * grid_size;
            element.y = (element.y / grid_size).round() * grid_size;
            element.width = (element.width / grid_size).round() * grid_size;
            element.height = (element.height / grid_size).round() * grid_size;
        }
        
        elements.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
        
        let mut current_y = self.spacing_rules.min_margin;
        for element in elements.iter_mut() {
            element.y = current_y;
            current_y += element.height + self.spacing_rules.element_gap;
        }
    }
    
    pub fn apply_emphasis(&self, elements: &mut Vec<DiagramElement>) {
        for element in elements.iter_mut() {
            if element.is_focal_point {
                element.border_width *= 1.5;
                element.shadow_intensity = 1.0;
                element.opacity = 1.0;
            } else {
                element.opacity = 0.85;
                element.shadow_intensity = 0.5;
            }
        }
    }
    
    pub fn apply_color_scheme(&self, elements: &mut Vec<DiagramElement>, scheme_name: &str) {
        if let Some(scheme) = self.color_schemes.get(scheme_name) {
            for element in elements.iter_mut() {
                element.fill_color = match element.element_type.as_str() {
                    "system" => scheme.primary.clone(),
                    "component" => scheme.secondary.clone(),
                    "function" => scheme.accent.clone(),
                    _ => scheme.background.clone(),
                };
                
                element.border_color = scheme.border.clone();
                element.text_color = scheme.text.clone();
                
                if element.use_gradient {
                    element.gradient_start = scheme.gradient_start.clone();
                    element.gradient_end = scheme.gradient_end.clone();
                }
            }
        }
    }
    
    pub fn apply_typography(&self, elements: &mut Vec<DiagramElement>) {
        let rules = &self.typography_rules;
        
        for element in elements.iter_mut() {
            match element.element_type.as_str() {
                "system" => {
                    element.font_family = rules.title_font.clone();
                    element.font_size = rules.title_size;
                    element.font_weight = "bold".to_string();
                }
                "component" | "function" => {
                    element.font_family = rules.label_font.clone();
                    element.font_size = rules.label_size;
                    element.font_weight = "normal".to_string();
                }
                _ => {
                    element.font_family = rules.annotation_font.clone();
                    element.font_size = rules.annotation_size;
                    element.font_weight = "normal".to_string();
                }
            }
            
            element.line_height = rules.line_height;
            element.letter_spacing = rules.letter_spacing;
        }
    }
    
    pub fn apply_spacing(&self, elements: &mut Vec<DiagramElement>) {
        let rules = &self.spacing_rules;
        
        for element in elements.iter_mut() {
            element.padding = rules.padding;
            element.margin = rules.min_margin;
        }
        
        for i in 0..elements.len() {
            for j in (i + 1)..elements.len() {
                let gap = self.calculate_gap(&elements[i], &elements[j]);
                if gap < rules.element_gap {
                    elements[j].y += rules.element_gap - gap;
                }
            }
        }
    }
    
    fn calculate_gap(&self, e1: &DiagramElement, e2: &DiagramElement) -> f64 {
        let x_overlap = (e1.x + e1.width).min(e2.x + e2.width) - e1.x.max(e2.x);
        let y_overlap = (e1.y + e1.height).min(e2.y + e2.height) - e1.y.max(e2.y);
        
        if x_overlap > 0.0 && y_overlap > 0.0 {
            0.0
        } else if x_overlap > 0.0 {
            (e2.y - (e1.y + e1.height)).abs()
        } else if y_overlap > 0.0 {
            (e2.x - (e1.x + e1.width)).abs()
        } else {
            self.calculate_distance(e1, e2)
        }
    }
    
    pub fn apply_polish(&self, elements: &mut Vec<DiagramElement>, settings: &PolishSettings) {
        for element in elements.iter_mut() {
            element.anti_aliasing = settings.anti_aliasing;
            element.smooth_curves = settings.smooth_curves;
            
            if settings.shadow_enabled {
                element.shadow_blur = settings.shadow_blur;
                element.shadow_offset_x = settings.shadow_offset_x;
                element.shadow_offset_y = settings.shadow_offset_y;
                element.shadow_color = settings.shadow_color.clone();
            }
            
            element.corner_radius = settings.corner_radius;
            element.render_dpi = settings.dpi;
        }
    }
    
    pub fn calculate_aesthetic_score(&self, elements: &[DiagramElement]) -> AestheticScore {
        let balance_score = self.calculate_balance_score(elements);
        let rhythm_score = self.calculate_rhythm_score(elements);
        let harmony_score = self.calculate_harmony_score(elements);
        let emphasis_score = self.calculate_emphasis_score(elements);
        let polish_score = self.calculate_polish_score(elements);
        
        let overall = (balance_score + rhythm_score + harmony_score + emphasis_score + polish_score) / 5.0;
        
        AestheticScore {
            overall,
            balance: balance_score,
            rhythm: rhythm_score,
            harmony: harmony_score,
            emphasis: emphasis_score,
            polish: polish_score,
        }
    }
    
    pub fn calculate_balance_score(&self, elements: &[DiagramElement]) -> f64 {
        let mut quadrant_weights = [0.0; 4];
        
        for element in elements {
            let quadrant = self.get_quadrant(element);
            quadrant_weights[quadrant] += element.visual_weight;
        }
        
        let mean = quadrant_weights.iter().sum::<f64>() / 4.0;
        let variance = quadrant_weights.iter()
            .map(|&w| (w - mean).powi(2))
            .sum::<f64>() / 4.0;
        
        1.0 - (variance.sqrt() / mean).min(1.0)
    }
    
    fn calculate_rhythm_score(&self, elements: &[DiagramElement]) -> f64 {
        let mut aligned_count = 0;
        let grid = self.spacing_rules.grid_size;
        
        for element in elements {
            if (element.x % grid).abs() < 1.0 && (element.y % grid).abs() < 1.0 {
                aligned_count += 1;
            }
        }
        
        aligned_count as f64 / elements.len().max(1) as f64
    }
    
    fn calculate_harmony_score(&self, elements: &[DiagramElement]) -> f64 {
        let mut color_variety = HashSet::new();
        for element in elements {
            color_variety.insert(&element.fill_color);
        }
        
        let variety_ratio = color_variety.len() as f64 / elements.len().max(1) as f64;
        1.0 - (variety_ratio - 0.2).abs().min(1.0)
    }
    
    fn calculate_emphasis_score(&self, elements: &[DiagramElement]) -> f64 {
        let focal_count = elements.iter().filter(|e| e.is_focal_point).count();
        let ideal_focal = (elements.len() as f64 * 0.15).max(1.0);
        
        1.0 - ((focal_count as f64 - ideal_focal) / ideal_focal).abs().min(1.0)
    }
    
    fn calculate_polish_score(&self, elements: &[DiagramElement]) -> f64 {
        let mut polished_count = 0;
        
        for element in elements {
            if element.anti_aliasing && element.smooth_curves && element.shadow_blur > 0.0 {
                polished_count += 1;
            }
        }
        
        polished_count as f64 / elements.len().max(1) as f64
    }
}

#[derive(Debug, Clone)]
pub struct DiagramElement {
    pub id: String,
    pub element_type: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub visual_weight: f64,
    pub size_multiplier: f64,
    pub is_focal_point: bool,
    pub group_id: Option<String>,
    pub proximity_group: Option<String>,
    pub fill_color: String,
    pub border_color: String,
    pub text_color: String,
    pub border_width: f64,
    pub shadow_intensity: f64,
    pub opacity: f64,
    pub use_gradient: bool,
    pub gradient_start: String,
    pub gradient_end: String,
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: String,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub padding: f64,
    pub margin: f64,
    pub anti_aliasing: bool,
    pub smooth_curves: bool,
    pub enable_smooth_edges: bool,
    pub edge_interpolation: EdgeInterpolation,
    pub shadow_blur: f64,
    pub shadow_offset_x: f64,
    pub shadow_offset_y: f64,
    pub shadow_color: String,
    pub corner_radius: f64,
    pub render_dpi: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeInterpolation {
    Linear,
    Bezier,
    Spline,
}

impl Default for DiagramElement {
    fn default() -> Self {
        Self {
            id: String::new(),
            element_type: String::new(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 60.0,
            visual_weight: 0.5,
            size_multiplier: 1.0,
            is_focal_point: false,
            group_id: None,
            proximity_group: None,
            fill_color: "#ECF0F1".to_string(),
            border_color: "#BDC3C7".to_string(),
            text_color: "#2C3E50".to_string(),
            border_width: 2.0,
            shadow_intensity: 0.5,
            opacity: 1.0,
            use_gradient: false,
            gradient_start: "#3498DB".to_string(),
            gradient_end: "#2980B9".to_string(),
            font_family: "Helvetica Neue, Arial, sans-serif".to_string(),
            font_size: 12.0,
            font_weight: "normal".to_string(),
            line_height: 1.5,
            letter_spacing: 0.0,
            padding: 12.0,
            margin: 20.0,
            anti_aliasing: true,
            smooth_curves: true,
            enable_smooth_edges: false,
            edge_interpolation: EdgeInterpolation::Linear,
            shadow_blur: 4.0,
            shadow_offset_x: 2.0,
            shadow_offset_y: 2.0,
            shadow_color: "rgba(0, 0, 0, 0.15)".to_string(),
            corner_radius: 6.0,
            render_dpi: 300,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AestheticScore {
    pub overall: f64,
    pub balance: f64,
    pub rhythm: f64,
    pub harmony: f64,
    pub emphasis: f64,
    pub polish: f64,
}

impl AestheticScore {
    pub fn print_report(&self) {
        println!("=== Aesthetic Quality Report ===");
        println!("Overall Score:  {:.2}%", self.overall * 100.0);
        println!("  Balance:      {:.2}%", self.balance * 100.0);
        println!("  Rhythm:       {:.2}%", self.rhythm * 100.0);
        println!("  Harmony:      {:.2}%", self.harmony * 100.0);
        println!("  Emphasis:     {:.2}%", self.emphasis * 100.0);
        println!("  Polish:       {:.2}%", self.polish * 100.0);
    }
}

pub fn apply_all_aesthetic_improvements(
    elements: &mut Vec<DiagramElement>,
    scheme: &str,
) -> AestheticScore {
    let aesthetic = AestheticIntelligence::new();
    let gestalt = GestaltPrinciples::default();
    let polish = PolishSettings::default();
    
    aesthetic.apply_visual_hierarchy(elements);
    aesthetic.apply_gestalt_principles(elements, &gestalt);
    aesthetic.apply_balance(elements);
    aesthetic.apply_rhythm(elements);
    aesthetic.apply_emphasis(elements);
    aesthetic.apply_color_scheme(elements, scheme);
    aesthetic.apply_typography(elements);
    aesthetic.apply_spacing(elements);
    aesthetic.apply_polish(elements, &polish);
    
    aesthetic.calculate_aesthetic_score(elements)
}
