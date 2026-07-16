use arclang::compiler::aesthetic_intelligence::*;

#[test]
fn test_color_schemes() {
    let aesthetic = AestheticIntelligence::new();
    
    let mut elements = vec![
        DiagramElement {
            id: "sys1".to_string(),
            element_type: "system".to_string(),
            x: 100.0,
            y: 100.0,
            ..Default::default()
        },
        DiagramElement {
            id: "comp1".to_string(),
            element_type: "component".to_string(),
            x: 150.0,
            y: 200.0,
            ..Default::default()
        },
    ];
    
    aesthetic.apply_color_scheme(&mut elements, "professional");
    
    assert_eq!(elements[0].fill_color, "#2C3E50");
    assert_eq!(elements[1].fill_color, "#34495E");
}

#[test]
fn test_visual_hierarchy() {
    let aesthetic = AestheticIntelligence::new();
    
    let mut elements = vec![
        DiagramElement {
            id: "sys1".to_string(),
            element_type: "system".to_string(),
            ..Default::default()
        },
        DiagramElement {
            id: "port1".to_string(),
            element_type: "port".to_string(),
            ..Default::default()
        },
    ];
    
    aesthetic.apply_visual_hierarchy(&mut elements);
    
    assert!(elements[0].size_multiplier > elements[1].size_multiplier);
    assert!(elements[0].is_focal_point);
    assert!(!elements[1].is_focal_point);
}

#[test]
fn test_gestalt_principles() {
    let aesthetic = AestheticIntelligence::new();
    let gestalt = GestaltPrinciples::default();
    
    let mut elements = vec![
        DiagramElement {
            id: "comp1".to_string(),
            element_type: "component".to_string(),
            x: 100.0,
            y: 100.0,
            ..Default::default()
        },
        DiagramElement {
            id: "comp2".to_string(),
            element_type: "component".to_string(),
            x: 120.0,
            y: 110.0,
            ..Default::default()
        },
    ];
    
    aesthetic.apply_gestalt_principles(&mut elements, &gestalt);
    
    assert!(elements[0].group_id.is_some());
    assert_eq!(elements[0].group_id, elements[1].group_id);
    assert!(elements[0].enable_smooth_edges);
}

#[test]
fn test_balance() {
    let aesthetic = AestheticIntelligence::new();
    
    let mut elements = vec![
        DiagramElement {
            id: "e1".to_string(),
            element_type: "system".to_string(),
            x: -100.0,
            y: -100.0,
            visual_weight: 1.0,
            ..Default::default()
        },
        DiagramElement {
            id: "e2".to_string(),
            element_type: "component".to_string(),
            x: 100.0,
            y: 100.0,
            visual_weight: 0.8,
            ..Default::default()
        },
    ];
    
    aesthetic.apply_balance(&mut elements);
    
    let score = aesthetic.calculate_balance_score(&elements);
    assert!(score >= 0.0 && score <= 1.0);
}

#[test]
fn test_rhythm() {
    let aesthetic = AestheticIntelligence::new();
    
    let mut elements = vec![
        DiagramElement {
            id: "e1".to_string(),
            element_type: "component".to_string(),
            x: 105.5,
            y: 203.2,
            width: 95.7,
            height: 61.3,
            ..Default::default()
        },
        DiagramElement {
            id: "e2".to_string(),
            element_type: "component".to_string(),
            x: 210.3,
            y: 407.8,
            width: 103.2,
            height: 58.9,
            ..Default::default()
        },
    ];
    
    aesthetic.apply_rhythm(&mut elements);
    
    let grid = 8.0;
    for element in &elements {
        assert!((element.x % grid).abs() < 0.1);
        assert!((element.width % grid).abs() < 0.1);
        assert!((element.height % grid).abs() < 0.1);
    }
    
    assert!(elements[0].y < elements[1].y);
}

#[test]
fn test_emphasis() {
    let aesthetic = AestheticIntelligence::new();
    
    let mut elements = vec![
        DiagramElement {
            id: "focal".to_string(),
            element_type: "system".to_string(),
            is_focal_point: true,
            border_width: 2.0,
            ..Default::default()
        },
        DiagramElement {
            id: "background".to_string(),
            element_type: "component".to_string(),
            is_focal_point: false,
            border_width: 2.0,
            ..Default::default()
        },
    ];
    
    aesthetic.apply_emphasis(&mut elements);
    
    assert!(elements[0].border_width > elements[1].border_width);
    assert_eq!(elements[0].opacity, 1.0);
    assert!(elements[1].opacity < 1.0);
}

#[test]
fn test_typography() {
    let aesthetic = AestheticIntelligence::new();
    
    let mut elements = vec![
        DiagramElement {
            id: "title".to_string(),
            element_type: "system".to_string(),
            ..Default::default()
        },
        DiagramElement {
            id: "label".to_string(),
            element_type: "component".to_string(),
            ..Default::default()
        },
    ];
    
    aesthetic.apply_typography(&mut elements);
    
    assert!(elements[0].font_size > elements[1].font_size);
    assert_eq!(elements[0].font_weight, "bold");
    assert_eq!(elements[1].font_weight, "normal");
}

#[test]
fn test_spacing() {
    let aesthetic = AestheticIntelligence::new();
    
    let mut elements = vec![
        DiagramElement {
            id: "e1".to_string(),
            element_type: "component".to_string(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 60.0,
            ..Default::default()
        },
    ];
    
    aesthetic.apply_spacing(&mut elements);
    
    assert_eq!(elements[0].padding, 12.0);
    assert_eq!(elements[0].margin, 20.0);
}

#[test]
fn test_polish() {
    let polish = PolishSettings::default();
    let aesthetic = AestheticIntelligence::new();
    
    let mut elements = vec![
        DiagramElement {
            id: "e1".to_string(),
            element_type: "component".to_string(),
            ..Default::default()
        },
    ];
    
    aesthetic.apply_polish(&mut elements, &polish);
    
    assert!(elements[0].anti_aliasing);
    assert!(elements[0].smooth_curves);
    assert_eq!(elements[0].shadow_blur, 4.0);
    assert_eq!(elements[0].corner_radius, 6.0);
    assert_eq!(elements[0].render_dpi, 300);
}

#[test]
fn test_aesthetic_score() {
    let aesthetic = AestheticIntelligence::new();
    
    let elements = vec![
        DiagramElement {
            id: "e1".to_string(),
            element_type: "system".to_string(),
            x: 0.0,
            y: 0.0,
            visual_weight: 1.0,
            is_focal_point: true,
            fill_color: "#2C3E50".to_string(),
            anti_aliasing: true,
            smooth_curves: true,
            shadow_blur: 4.0,
            ..Default::default()
        },
        DiagramElement {
            id: "e2".to_string(),
            element_type: "component".to_string(),
            x: 8.0,
            y: 80.0,
            visual_weight: 0.8,
            is_focal_point: false,
            fill_color: "#2C3E50".to_string(),
            anti_aliasing: true,
            smooth_curves: true,
            shadow_blur: 4.0,
            ..Default::default()
        },
    ];
    
    let score = aesthetic.calculate_aesthetic_score(&elements);
    
    assert!(score.overall >= 0.0 && score.overall <= 1.0);
    assert!(score.balance >= 0.0 && score.balance <= 1.0);
    assert!(score.rhythm >= 0.0 && score.rhythm <= 1.0);
    assert!(score.harmony >= 0.0 && score.harmony <= 1.0);
    assert!(score.emphasis >= 0.0 && score.emphasis <= 1.0);
    assert!(score.polish >= 0.0 && score.polish <= 1.0);
}

#[test]
fn test_apply_all_aesthetic_improvements() {
    let mut elements = vec![
        DiagramElement {
            id: "sys1".to_string(),
            element_type: "system".to_string(),
            x: 105.5,
            y: 103.2,
            ..Default::default()
        },
        DiagramElement {
            id: "comp1".to_string(),
            element_type: "component".to_string(),
            x: 208.3,
            y: 207.8,
            ..Default::default()
        },
        DiagramElement {
            id: "func1".to_string(),
            element_type: "function".to_string(),
            x: 312.7,
            y: 305.1,
            ..Default::default()
        },
    ];
    
    let score = apply_all_aesthetic_improvements(&mut elements, "professional");
    
    assert!(elements[0].fill_color == "#2C3E50");
    assert!(elements[0].is_focal_point);
    assert!(elements[0].anti_aliasing);
    assert_eq!(elements[0].render_dpi, 300);
    
    assert!(score.overall >= 0.0 && score.overall <= 1.0);
    assert!(score.polish > 0.8);
    
    score.print_report();
}

#[test]
fn test_multiple_color_schemes() {
    let aesthetic = AestheticIntelligence::new();
    let mut elements = vec![
        DiagramElement {
            id: "e1".to_string(),
            element_type: "system".to_string(),
            ..Default::default()
        },
    ];
    
    aesthetic.apply_color_scheme(&mut elements, "elegant");
    assert_eq!(elements[0].fill_color, "#37474F");
    
    aesthetic.apply_color_scheme(&mut elements, "modern");
    assert_eq!(elements[0].fill_color, "#006064");
    
    aesthetic.apply_color_scheme(&mut elements, "vibrant");
    assert_eq!(elements[0].fill_color, "#4A148C");
}

#[test]
fn test_proximity_grouping() {
    let aesthetic = AestheticIntelligence::new();
    let gestalt = GestaltPrinciples {
        proximity_threshold: 100.0,
        similarity_grouping: false,
        continuity_enabled: false,
        closure_enabled: false,
    };
    
    let mut elements = vec![
        DiagramElement {
            id: "e1".to_string(),
            element_type: "component".to_string(),
            x: 0.0,
            y: 0.0,
            ..Default::default()
        },
        DiagramElement {
            id: "e2".to_string(),
            element_type: "function".to_string(),
            x: 50.0,
            y: 50.0,
            ..Default::default()
        },
        DiagramElement {
            id: "e3".to_string(),
            element_type: "component".to_string(),
            x: 500.0,
            y: 500.0,
            ..Default::default()
        },
    ];
    
    aesthetic.apply_gestalt_principles(&mut elements, &gestalt);
    
    assert_eq!(elements[0].proximity_group, elements[1].proximity_group);
    assert_ne!(elements[0].proximity_group, elements[2].proximity_group);
}

#[test]
fn test_edge_interpolation() {
    let aesthetic = AestheticIntelligence::new();
    let gestalt = GestaltPrinciples {
        proximity_threshold: 50.0,
        similarity_grouping: false,
        continuity_enabled: true,
        closure_enabled: false,
    };
    
    let mut elements = vec![
        DiagramElement {
            id: "e1".to_string(),
            element_type: "component".to_string(),
            ..Default::default()
        },
    ];
    
    aesthetic.apply_gestalt_principles(&mut elements, &gestalt);
    
    assert_eq!(elements[0].edge_interpolation, EdgeInterpolation::Bezier);
}
