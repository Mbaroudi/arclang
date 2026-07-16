"""
MBSE Capella Expert System for requirement analysis and diagram enhancement.
"""

from typing import Dict, List, Any, Optional
from pathlib import Path
import re
import json


class MBSECapellaExpert:
    """Expert system for analyzing requirements and enhancing Capella diagrams."""
    
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        
        # Capella methodology knowledge base
        self.arcadia_layers = {
            "operational": {
                "name": "Operational Analysis",
                "focus": "What users need to do",
                "stakeholders": ["end_users", "operators", "maintainers"],
                "key_elements": ["actors", "operational_activities", "entity", "operational_capabilities"],
                "diagram_types": ["operational", "capability"]
            },
            "system": {
                "name": "System Analysis", 
                "focus": "What the system must do",
                "stakeholders": ["system_engineers", "customers"],
                "key_elements": ["system_functions", "system_actors", "capabilities", "missions"],
                "diagram_types": ["functional", "capability", "tree"]
            },
            "logical": {
                "name": "Logical Architecture",
                "focus": "How the system is structured",
                "stakeholders": ["architects", "designers"],
                "key_elements": ["logical_components", "interfaces", "component_exchanges"],
                "diagram_types": ["component", "sequence", "state-machine"]
            },
            "physical": {
                "name": "Physical Architecture",
                "focus": "How to build the system",
                "stakeholders": ["hardware_engineers", "integration_teams"],
                "key_elements": ["behavior_components", "hardware_components", "nodes", "deployment"],
                "diagram_types": ["physical", "component"]
            },
            "epbs": {
                "name": "End Product Breakdown Structure",
                "focus": "What to build/buy",
                "stakeholders": ["procurement", "manufacturing"],
                "key_elements": ["configuration_items", "hwci", "csci"],
                "diagram_types": ["tree", "class"]
            }
        }
        
        # Diagram enhancement patterns
        self.enhancement_patterns = {
            "operational": {
                "required_elements": ["actors", "activities", "swimlanes"],
                "best_practices": [
                    "Use stick figures for human actors",
                    "Group activities by actor in swimlanes",
                    "Show protocol labels (CAN, V2X, HMI)",
                    "Include operational capabilities"
                ],
                "quality_checks": ["has_actors", "has_activities", "has_swimlanes"]
            },
            "functional": {
                "required_elements": ["functions", "functional_exchanges", "ports"],
                "best_practices": [
                    "Use category colors (7 types)",
                    "Show data flow direction",
                    "Include external actor boundaries",
                    "Label exchange types"
                ],
                "quality_checks": ["has_functions", "has_exchanges", "has_ports"]
            },
            "component": {
                "required_elements": ["components", "interfaces", "component_exchanges"],
                "best_practices": [
                    "Use UML lollipop notation for provided interfaces",
                    "Use socket notation for required interfaces",
                    "Show interface protocols (CAN, Ethernet, SPI)",
                    "Apply Capella colors (green sensors, blue controllers, orange actuators)",
                    "Add ASIL borders for safety-critical components"
                ],
                "quality_checks": ["has_interfaces", "has_protocols", "has_safety_borders"]
            },
            "sequence": {
                "required_elements": ["participants", "messages", "lifelines"],
                "best_practices": [
                    "Time-ordered messages top to bottom",
                    "Use fragment blocks (alt, loop, opt)",
                    "Distinguish sync vs async calls",
                    "Include return messages"
                ],
                "quality_checks": ["has_lifelines", "has_messages", "proper_ordering"]
            },
            "state-machine": {
                "required_elements": ["states", "transitions", "events"],
                "best_practices": [
                    "Show initial and final states",
                    "Label transitions with events/guards",
                    "Include entry/exit actions",
                    "Group related states in regions"
                ],
                "quality_checks": ["has_initial_state", "has_transitions", "has_guards"]
            },
            "physical": {
                "required_elements": ["nodes", "behavior_components", "deployment"],
                "best_practices": [
                    "Use 3D ECU boxes with gold color (#FFE699)",
                    "Nest behavior_components inside nodes",
                    "Show allocated functions",
                    "Include processor/memory specs",
                    "Display communication buses"
                ],
                "quality_checks": ["has_3d_boxes", "has_nested_components", "has_deployment"]
            },
            "class": {
                "required_elements": ["classes", "attributes", "operations"],
                "best_practices": [
                    "UML class notation",
                    "Show inheritance hierarchies",
                    "Include associations and multiplicities",
                    "Separate interface from implementation"
                ],
                "quality_checks": ["has_classes", "has_attributes", "has_operations"]
            },
            "tree": {
                "required_elements": ["nodes", "hierarchy"],
                "best_practices": [
                    "Use Reingold-Tilford layout",
                    "Show expand/collapse indicators",
                    "Apply category icons",
                    "Balance tree depth"
                ],
                "quality_checks": ["has_hierarchy", "balanced_depth"]
            },
            "capability": {
                "required_elements": ["capabilities", "missions"],
                "best_practices": [
                    "3-level hierarchy (Mission/Capability/Operational)",
                    "Color-code by level",
                    "Show capability associations",
                    "Link to operational activities"
                ],
                "quality_checks": ["has_3_levels", "has_associations"]
            },
            "functional-chain": {
                "required_elements": ["functions", "exchanges"],
                "best_practices": [
                    "Left-to-right execution flow",
                    "Show function sequence clearly",
                    "Label data exchanges",
                    "Highlight critical paths"
                ],
                "quality_checks": ["has_sequence", "has_exchanges", "clear_flow"]
            }
        }
    
    def analyze_requirements(self, requirements_text: str) -> Dict[str, Any]:
        """
        Analyze natural language requirements and map to Arcadia layers.
        
        Returns recommended architecture structure.
        """
        analysis = {
            "requirement_count": 0,
            "arcadia_mapping": {},
            "recommended_components": [],
            "recommended_diagrams": [],
            "safety_requirements": [],
            "traceability_plan": {}
        }
        
        # Split into individual requirements
        req_lines = [line.strip() for line in requirements_text.split('\n') if line.strip()]
        analysis["requirement_count"] = len(req_lines)
        
        # Keywords for layer identification
        operational_keywords = ["user shall", "operator shall", "driver shall", "maintenance", "operational capability"]
        system_keywords = ["system shall", "vehicle shall", "function", "capability", "mission"]
        logical_keywords = ["component shall", "interface", "protocol", "exchange", "communication"]
        physical_keywords = ["hardware", "ECU", "processor", "deployment", "node", "sensor", "actuator"]
        safety_keywords = ["safety", "ASIL", "fail-safe", "redundant", "critical", "hazard", "SIL", "DAL"]
        
        # Analyze each requirement
        for req in req_lines:
            req_lower = req.lower()
            
            # Identify Arcadia layer
            if any(kw in req_lower for kw in operational_keywords):
                analysis["arcadia_mapping"].setdefault("operational", []).append(req)
            elif any(kw in req_lower for kw in physical_keywords):
                analysis["arcadia_mapping"].setdefault("physical", []).append(req)
            elif any(kw in req_lower for kw in logical_keywords):
                analysis["arcadia_mapping"].setdefault("logical", []).append(req)
            elif any(kw in req_lower for kw in system_keywords):
                analysis["arcadia_mapping"].setdefault("system", []).append(req)
            else:
                analysis["arcadia_mapping"].setdefault("system", []).append(req)
            
            # Identify safety requirements
            if any(kw in req_lower for kw in safety_keywords):
                asil_level = "ASIL_D" if "asil-d" in req_lower or "asil d" in req_lower else \
                            "ASIL_C" if "asil-c" in req_lower or "asil c" in req_lower else \
                            "ASIL_B" if "asil-b" in req_lower or "asil b" in req_lower else "ASIL_A"
                analysis["safety_requirements"].append({
                    "requirement": req,
                    "level": asil_level
                })
            
            # Extract component hints
            if "sensor" in req_lower:
                analysis["recommended_components"].append({"type": "sensor", "hint": req})
            if "controller" in req_lower or "control" in req_lower:
                analysis["recommended_components"].append({"type": "controller", "hint": req})
            if "actuator" in req_lower:
                analysis["recommended_components"].append({"type": "actuator", "hint": req})
        
        # Recommend diagram types based on identified layers
        for layer, reqs in analysis["arcadia_mapping"].items():
            if layer in self.arcadia_layers:
                for diagram_type in self.arcadia_layers[layer]["diagram_types"]:
                    if diagram_type not in analysis["recommended_diagrams"]:
                        analysis["recommended_diagrams"].append(diagram_type)
        
        # Generate traceability plan
        analysis["traceability_plan"] = {
            "operational_to_system": len(analysis["arcadia_mapping"].get("operational", [])),
            "system_to_logical": len(analysis["arcadia_mapping"].get("system", [])),
            "logical_to_physical": len(analysis["arcadia_mapping"].get("logical", [])),
            "safety_traces_needed": len(analysis["safety_requirements"])
        }
        
        return analysis
    
    def generate_arclang_skeleton(self, analysis: Dict[str, Any], project_name: str) -> str:
        """
        Generate ArcLang skeleton from requirement analysis.
        
        Returns complete .arc file content.
        """
        arc_content = f'model "{project_name}" {{\n\n'
        
        # Generate requirements block
        arc_content += '    requirements stakeholder {\n'
        for i, req in enumerate(analysis["arcadia_mapping"].get("operational", [])[:5], 1):
            req_id = f"STK-{i:03d}"
            req_title = req[:50] + "..." if len(req) > 50 else req
            arc_content += f'        req {req_id} "{req_title}" {{\n'
            arc_content += f'            description: "{req}"\n'
            arc_content += '            priority: "High"\n'
            arc_content += '        }\n'
        arc_content += '    }\n\n'
        
        # Generate system requirements
        arc_content += '    requirements system {\n'
        for i, req in enumerate(analysis["arcadia_mapping"].get("system", [])[:5], 1):
            req_id = f"SYS-{i:03d}"
            req_title = req[:50] + "..." if len(req) > 50 else req
            arc_content += f'        req {req_id} "{req_title}" {{\n'
            arc_content += f'            description: "{req}"\n'
            stk_idx = min(i, len(analysis["arcadia_mapping"].get("operational", [])))
            arc_content += f'            traces: ["STK-{stk_idx:03d}"]\n'
            arc_content += '        }\n'
        arc_content += '    }\n\n'
        
        # Generate safety requirements if any
        if analysis["safety_requirements"]:
            arc_content += '    requirements safety {\n'
            for i, safe_req in enumerate(analysis["safety_requirements"][:5], 1):
                req_id = f"SAF-{i:03d}"
                req_title = safe_req["requirement"][:50] + "..." if len(safe_req["requirement"]) > 50 else safe_req["requirement"]
                arc_content += f'        req {req_id} "{req_title}" {{\n'
                arc_content += f'            description: "{safe_req["requirement"]}"\n'
                arc_content += f'            safety_level: "{safe_req["level"]}"\n'
                arc_content += '        }\n'
            arc_content += '    }\n\n'
        
        # Generate operational architecture if operational requirements exist
        if analysis["arcadia_mapping"].get("operational"):
            arc_content += '    architecture operational {\n'
            arc_content += '        actor "User" {\n'
            arc_content += '            type: "human"\n'
            arc_content += '        }\n'
            arc_content += '        actor "System" {\n'
            arc_content += '            type: "system"\n'
            arc_content += '        }\n'
            arc_content += '    }\n\n'
        
        # Generate logical architecture
        arc_content += '    architecture logical {\n'
        
        # Add components based on analysis
        for i, comp_hint in enumerate(analysis["recommended_components"][:5], 1):
            comp_type = comp_hint["type"]
            comp_name = f"{comp_type.title()}Component{i}"
            color = "#70AD47" if comp_type == "sensor" else \
                   "#6495ED" if comp_type == "controller" else "#ED7D31"
            stereotype = f"<<{comp_type}>>"
            
            arc_content += f'        component {comp_name} "{comp_name}" {{\n'
            arc_content += f'            color: "{color}"\n'
            arc_content += f'            stereotype: "{stereotype}"\n'
            
            # Add interfaces based on type
            if comp_type == "sensor":
                arc_content += '            provides "SensorData" {\n'
                arc_content += '                protocol: "CAN"\n'
                arc_content += '            }\n'
            elif comp_type == "controller":
                arc_content += '            requires "SensorInput" {\n'
                arc_content += '                protocol: "CAN"\n'
                arc_content += '            }\n'
                arc_content += '            provides "ControlOutput" {\n'
                arc_content += '                protocol: "CAN"\n'
                arc_content += '            }\n'
            elif comp_type == "actuator":
                arc_content += '            requires "CommandInput" {\n'
                arc_content += '                protocol: "CAN"\n'
                arc_content += '            }\n'
            
            # Add safety level if applicable
            if analysis["safety_requirements"]:
                arc_content += '            safety_level: "ASIL_D"\n'
            
            arc_content += '        }\n'
        
        arc_content += '    }\n\n'
        
        # Generate physical architecture if physical requirements exist
        if analysis["arcadia_mapping"].get("physical"):
            arc_content += '    architecture physical {\n'
            arc_content += '        node "MainECU" {\n'
            arc_content += '            type: "ECU"\n'
            arc_content += '            processor: "ARM Cortex-A53"\n'
            arc_content += '            memory: "4GB RAM"\n'
            arc_content += '        }\n'
            arc_content += '    }\n\n'
        
        arc_content += '}\n'
        
        return arc_content
    
    def assess_diagram_quality(self, diagram_path: Path, diagram_type: str) -> Dict[str, Any]:
        """
        Assess diagram quality against Capella best practices.
        
        Returns quality score and improvement suggestions.
        """
        if not diagram_path.exists():
            return {"error": "Diagram not found"}
        
        content = diagram_path.read_text()
        
        # Get expected patterns for this diagram type
        patterns = self.enhancement_patterns.get(diagram_type, {})
        quality_checks = patterns.get("quality_checks", [])
        best_practices = patterns.get("best_practices", [])
        
        assessment = {
            "diagram_type": diagram_type,
            "quality_score": 0,
            "max_score": len(quality_checks) * 10,
            "checks_passed": [],
            "checks_failed": [],
            "improvements": []
        }
        
        # Perform quality checks
        if diagram_type == "component":
            # Check for interfaces
            if 'circle' in content and 'stroke="#FFFFFF"' in content:
                assessment["checks_passed"].append("has_interfaces")
                assessment["quality_score"] += 10
            else:
                assessment["checks_failed"].append("has_interfaces")
                assessment["improvements"].append("Add interface notation (lollipops for provides, sockets for requires)")
            
            # Check for protocols
            if any(proto in content for proto in ["CAN", "Ethernet", "SPI", "I2C"]):
                assessment["checks_passed"].append("has_protocols")
                assessment["quality_score"] += 10
            else:
                assessment["checks_failed"].append("has_protocols")
                assessment["improvements"].append("Add protocol labels to interfaces (CAN, Ethernet, etc.)")
            
            # Check for safety borders
            if 'stroke-width="6"' in content or "#8B0000" in content:
                assessment["checks_passed"].append("has_safety_borders")
                assessment["quality_score"] += 10
            else:
                assessment["checks_failed"].append("has_safety_borders")
                assessment["improvements"].append("Add ASIL borders (6px dark red) for safety-critical components")
        
        elif diagram_type == "physical":
            # Check for 3D effects
            if "drop-shadow" in content:
                assessment["checks_passed"].append("has_3d_boxes")
                assessment["quality_score"] += 10
            else:
                assessment["checks_failed"].append("has_3d_boxes")
                assessment["improvements"].append("Add 3D effects to ECU boxes with drop shadows")
            
            # Check for nested components
            if content.count("<g ") > 3:
                assessment["checks_passed"].append("has_nested_components")
                assessment["quality_score"] += 10
            else:
                assessment["checks_failed"].append("has_nested_components")
                assessment["improvements"].append("Add nested behavior_components inside ECU nodes")
            
            # Check for deployment info
            if any(word in content for word in ["processor", "memory", "allocated"]):
                assessment["checks_passed"].append("has_deployment")
                assessment["quality_score"] += 10
            else:
                assessment["checks_failed"].append("has_deployment")
                assessment["improvements"].append("Add deployment details (processor, memory specs)")
        
        elif diagram_type == "operational":
            # Check for actors
            if "actor" in content.lower() or "stick" in content.lower():
                assessment["checks_passed"].append("has_actors")
                assessment["quality_score"] += 10
            else:
                assessment["checks_failed"].append("has_actors")
                assessment["improvements"].append("Add actors (stick figures for humans, boxes for systems)")
            
            # Check for activities
            if "activity" in content.lower() or "⊕" in content:
                assessment["checks_passed"].append("has_activities")
                assessment["quality_score"] += 10
            else:
                assessment["checks_failed"].append("has_activities")
                assessment["improvements"].append("Add operational activities with ⊕ symbols")
        
        # Calculate percentage
        assessment["quality_percentage"] = int((assessment["quality_score"] / max(assessment["max_score"], 1)) * 100)
        
        # Add best practices not yet implemented
        for practice in best_practices[:3]:
            if practice not in assessment["improvements"]:
                assessment["improvements"].append(f"Best practice: {practice}")
        
        return assessment
    
    def suggest_enhancements(self, assessment: Dict[str, Any]) -> List[str]:
        """
        Suggest specific enhancements based on quality assessment.
        
        Returns list of enhancement actions.
        """
        enhancements = []
        
        diagram_type = assessment.get("diagram_type")
        
        # Generic enhancements
        if assessment.get("quality_percentage", 0) < 100:
            enhancements.append("increase_spacing")
            enhancements.append("enlarge_text")
        
        # Type-specific enhancements
        if diagram_type == "component":
            if "has_interfaces" in assessment.get("checks_failed", []):
                enhancements.append("add_interface_notation")
            if "has_safety_borders" in assessment.get("checks_failed", []):
                enhancements.append("add_safety_borders")
        
        elif diagram_type == "physical":
            if "has_3d_boxes" in assessment.get("checks_failed", []):
                enhancements.append("add_3d_effects")
            if "has_nested_components" in assessment.get("checks_failed", []):
                enhancements.append("add_nested_components")
        
        # Always good to improve
        enhancements.append("improve_contrast")
        
        return enhancements[:5]  # Limit to 5 enhancements
    
    def generate_mbse_report(self, analysis: Dict[str, Any], assessments: List[Dict[str, Any]]) -> str:
        """
        Generate comprehensive MBSE quality report.
        
        Returns markdown report.
        """
        report = "# MBSE Capella Architecture Analysis Report\n\n"
        
        # Requirements section
        report += "## Requirements Analysis\n\n"
        report += f"**Total Requirements**: {analysis.get('requirement_count', 0)}\n\n"
        
        report += "### Arcadia Layer Mapping\n\n"
        for layer, reqs in analysis.get("arcadia_mapping", {}).items():
            layer_info = self.arcadia_layers.get(layer, {})
            report += f"- **{layer_info.get('name', layer.title())}**: {len(reqs)} requirements\n"
            report += f"  - Focus: {layer_info.get('focus', 'N/A')}\n"
        
        report += "\n### Safety Requirements\n\n"
        if analysis.get("safety_requirements"):
            for safe_req in analysis["safety_requirements"]:
                report += f"- **{safe_req['level']}**: {safe_req['requirement'][:80]}...\n"
        else:
            report += "No safety requirements identified.\n"
        
        report += "\n### Recommended Diagrams\n\n"
        for diagram_type in analysis.get("recommended_diagrams", []):
            report += f"- {diagram_type}\n"
        
        # Diagram quality section
        if assessments:
            report += "\n## Diagram Quality Assessment\n\n"
            
            total_score = sum(a.get("quality_score", 0) for a in assessments)
            max_score = sum(a.get("max_score", 1) for a in assessments)
            overall_percentage = int((total_score / max(max_score, 1)) * 100)
            
            report += f"**Overall Quality Score**: {overall_percentage}%\n\n"
            
            for assessment in assessments:
                diagram_type = assessment.get("diagram_type")
                quality = assessment.get("quality_percentage", 0)
                
                report += f"### {diagram_type.title()} Diagram\n\n"
                report += f"- **Quality**: {quality}%\n"
                report += f"- **Checks Passed**: {len(assessment.get('checks_passed', []))}\n"
                report += f"- **Checks Failed**: {len(assessment.get('checks_failed', []))}\n\n"
                
                if assessment.get("improvements"):
                    report += "**Improvements**:\n"
                    for improvement in assessment["improvements"][:5]:
                        report += f"- {improvement}\n"
                    report += "\n"
        
        # Traceability section
        report += "## Traceability Plan\n\n"
        trace_plan = analysis.get("traceability_plan", {})
        report += f"- Operational → System: {trace_plan.get('operational_to_system', 0)} traces needed\n"
        report += f"- System → Logical: {trace_plan.get('system_to_logical', 0)} traces needed\n"
        report += f"- Logical → Physical: {trace_plan.get('logical_to_physical', 0)} traces needed\n"
        report += f"- Safety Traces: {trace_plan.get('safety_traces_needed', 0)} traces needed\n\n"
        
        report += "---\n\n"
        report += "*Generated by ArcLang MCP MBSE Expert System*\n"
        
        return report
