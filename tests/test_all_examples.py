#!/usr/bin/env python3
"""
Comprehensive Selenium-based test suite for ArcLang
Tests all examples with all visualization formats
"""

import os
import sys
import subprocess
import time
from pathlib import Path
from typing import List, Dict, Tuple
from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from dataclasses import dataclass
import json

@dataclass
class TestResult:
    name: str
    status: str  # 'pass', 'fail', 'skip'
    message: str
    duration: float
    file_size: int = 0

class ArcLangTester:
    def __init__(self, arclang_bin: str, examples_dir: str, output_dir: str):
        self.arclang_bin = arclang_bin
        self.examples_dir = Path(examples_dir)
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
        # Test configurations
        self.viz_formats = [
            'arc-viz-ultimate',
            'arc-viz-smart',
            'arc-viz-channel',
            'arc-viz-perfect',
            'arc-viz-elk',
        ]
        
        self.export_formats = [
            'mermaid',
            'plant-uml',
            'json',
            'capella',
        ]
        
        self.results: List[TestResult] = []
        
        # Setup Selenium
        self.setup_driver()
    
    def setup_driver(self):
        """Setup headless Chrome driver"""
        chrome_options = Options()
        chrome_options.add_argument('--headless')
        chrome_options.add_argument('--no-sandbox')
        chrome_options.add_argument('--disable-dev-shm-usage')
        chrome_options.add_argument('--disable-gpu')
        chrome_options.add_argument('--window-size=1920,1080')
        
        try:
            self.driver = webdriver.Chrome(options=chrome_options)
            print("✓ Selenium WebDriver initialized")
        except Exception as e:
            print(f"✗ Failed to initialize WebDriver: {e}")
            print("  Please install chromedriver: brew install chromedriver")
            sys.exit(1)
    
    def find_all_examples(self) -> List[Path]:
        """Find all .arc files in examples directory"""
        return list(self.examples_dir.rglob("*.arc"))
    
    def test_compilation(self, arc_file: Path) -> TestResult:
        """Test basic compilation"""
        start = time.time()
        name = f"compile:{arc_file.stem}"
        
        try:
            result = subprocess.run(
                [self.arclang_bin, 'build', str(arc_file)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            duration = time.time() - start
            
            if result.returncode == 0:
                return TestResult(name, 'pass', 'Compilation successful', duration)
            else:
                return TestResult(name, 'fail', f'Compilation failed: {result.stderr[:200]}', duration)
        
        except subprocess.TimeoutExpired:
            return TestResult(name, 'fail', 'Compilation timeout (30s)', 30.0)
        except Exception as e:
            return TestResult(name, 'fail', f'Compilation error: {str(e)}', time.time() - start)
    
    def test_viz_export(self, arc_file: Path, format: str) -> TestResult:
        """Test visualization export"""
        start = time.time()
        name = f"viz:{arc_file.stem}:{format}"
        output_file = self.output_dir / f"{arc_file.stem}_{format}.html"
        
        try:
            result = subprocess.run(
                [self.arclang_bin, 'export', str(arc_file), 
                 '-o', str(output_file), '-f', format],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            duration = time.time() - start
            
            if result.returncode != 0:
                return TestResult(name, 'fail', f'Export failed: {result.stderr[:200]}', duration)
            
            if not output_file.exists():
                return TestResult(name, 'fail', 'Output file not created', duration)
            
            file_size = output_file.stat().st_size
            
            if file_size < 100:
                return TestResult(name, 'fail', f'Output too small: {file_size} bytes', duration, file_size)
            
            return TestResult(name, 'pass', f'Export successful ({file_size} bytes)', duration, file_size)
        
        except subprocess.TimeoutExpired:
            return TestResult(name, 'fail', 'Export timeout (60s)', 60.0)
        except Exception as e:
            return TestResult(name, 'fail', f'Export error: {str(e)}', time.time() - start)
    
    def test_explorer(self, arc_file: Path) -> TestResult:
        """Test explorer generation and rendering"""
        start = time.time()
        name = f"explorer:{arc_file.stem}"
        output_file = arc_file.parent / f"{arc_file.stem}_explorer.html"
        
        try:
            # Generate explorer
            result = subprocess.run(
                [self.arclang_bin, 'explorer', str(arc_file)],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            if result.returncode != 0:
                return TestResult(name, 'fail', f'Explorer generation failed: {result.stderr[:200]}', 
                                time.time() - start)
            
            if not output_file.exists():
                return TestResult(name, 'fail', 'Explorer HTML not created', time.time() - start)
            
            # Test with Selenium
            try:
                self.driver.get(f"file://{output_file.absolute()}")
                
                # Wait for SVG or diagram to load (max 10 seconds)
                WebDriverWait(self.driver, 10).until(
                    EC.presence_of_element_located((By.TAG_NAME, "svg"))
                )
                
                # Check for errors in console
                logs = self.driver.get_log('browser')
                errors = [log for log in logs if log['level'] == 'SEVERE']
                
                # Filter out ELK fallback errors (non-critical - dagre renders successfully)
                critical_errors = [e for e in errors if 'ELK layout failed' not in e['message']]
                
                if critical_errors:
                    error_msg = '; '.join([e['message'][:100] for e in critical_errors[:3]])
                    return TestResult(name, 'fail', f'Browser errors: {error_msg}', 
                                    time.time() - start)
                
                # Check SVG dimensions
                svg = self.driver.find_element(By.TAG_NAME, "svg")
                width = svg.get_attribute("width")
                height = svg.get_attribute("height")
                
                file_size = output_file.stat().st_size
                
                return TestResult(name, 'pass', 
                                f'Explorer rendered successfully (SVG: {width}x{height}, {file_size} bytes)', 
                                time.time() - start, file_size)
            
            except Exception as e:
                return TestResult(name, 'fail', f'Selenium test failed: {str(e)}', 
                                time.time() - start)
        
        except subprocess.TimeoutExpired:
            return TestResult(name, 'fail', 'Explorer generation timeout (60s)', 60.0)
        except Exception as e:
            return TestResult(name, 'fail', f'Explorer error: {str(e)}', time.time() - start)
    
    def test_other_exports(self, arc_file: Path, format: str) -> TestResult:
        """Test other export formats (Mermaid, PlantUML, JSON, etc.)"""
        start = time.time()
        name = f"export:{arc_file.stem}:{format}"
        
        ext_map = {
            'mermaid': '.mmd',
            'plant-uml': '.puml',
            'json': '.json',
            'capella': '.xml',
        }
        
        ext = ext_map.get(format, '.txt')
        output_file = self.output_dir / f"{arc_file.stem}_{format}{ext}"
        
        try:
            result = subprocess.run(
                [self.arclang_bin, 'export', str(arc_file), 
                 '-o', str(output_file), '-f', format],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            duration = time.time() - start
            
            if result.returncode != 0:
                return TestResult(name, 'fail', f'Export failed: {result.stderr[:200]}', duration)
            
            if not output_file.exists():
                return TestResult(name, 'fail', 'Output file not created', duration)
            
            file_size = output_file.stat().st_size
            
            if file_size < 10:
                return TestResult(name, 'fail', f'Output too small: {file_size} bytes', duration, file_size)
            
            return TestResult(name, 'pass', f'Export successful ({file_size} bytes)', duration, file_size)
        
        except subprocess.TimeoutExpired:
            return TestResult(name, 'fail', 'Export timeout (30s)', 30.0)
        except Exception as e:
            return TestResult(name, 'fail', f'Export error: {str(e)}', time.time() - start)
    
    def run_all_tests(self):
        """Run comprehensive test suite"""
        examples = self.find_all_examples()
        
        print(f"\n{'='*80}")
        print(f"ArcLang Comprehensive Test Suite")
        print(f"{'='*80}")
        print(f"Found {len(examples)} example files")
        print(f"Testing {len(self.viz_formats)} visualization formats")
        print(f"Testing {len(self.export_formats)} export formats")
        print(f"Output directory: {self.output_dir}")
        print(f"{'='*80}\n")
        
        total_tests = len(examples) * (1 + len(self.viz_formats) + 1 + len(self.export_formats))
        current_test = 0
        
        for arc_file in examples:
            print(f"\n{'─'*80}")
            print(f"Testing: {arc_file.relative_to(self.examples_dir)}")
            print(f"{'─'*80}")
            
            # 1. Test compilation
            current_test += 1
            print(f"[{current_test}/{total_tests}] Compiling...")
            result = self.test_compilation(arc_file)
            self.results.append(result)
            self.print_result(result)
            
            if result.status == 'fail':
                print(f"  ⚠ Skipping other tests for {arc_file.stem} due to compilation failure")
                continue
            
            # 2. Test all viz formats
            for viz_format in self.viz_formats:
                current_test += 1
                print(f"[{current_test}/{total_tests}] Testing viz format: {viz_format}...")
                result = self.test_viz_export(arc_file, viz_format)
                self.results.append(result)
                self.print_result(result)
            
            # 3. Test explorer
            current_test += 1
            print(f"[{current_test}/{total_tests}] Testing explorer...")
            result = self.test_explorer(arc_file)
            self.results.append(result)
            self.print_result(result)
            
            # 4. Test other export formats
            for export_format in self.export_formats:
                current_test += 1
                print(f"[{current_test}/{total_tests}] Testing export format: {export_format}...")
                result = self.test_other_exports(arc_file, export_format)
                self.results.append(result)
                self.print_result(result)
        
        self.cleanup()
    
    def print_result(self, result: TestResult):
        """Print test result"""
        status_emoji = {
            'pass': '✓',
            'fail': '✗',
            'skip': '⊘'
        }
        
        emoji = status_emoji.get(result.status, '?')
        print(f"  {emoji} {result.name}: {result.message} ({result.duration:.2f}s)")
    
    def generate_report(self) -> str:
        """Generate comprehensive test report"""
        total = len(self.results)
        passed = sum(1 for r in self.results if r.status == 'pass')
        failed = sum(1 for r in self.results if r.status == 'fail')
        skipped = sum(1 for r in self.results if r.status == 'skip')
        
        pass_rate = (passed / total * 100) if total > 0 else 0
        
        report = f"""
{'='*80}
ArcLang Test Report
{'='*80}

Summary:
  Total Tests:    {total}
  Passed:         {passed} ({pass_rate:.1f}%)
  Failed:         {failed}
  Skipped:        {skipped}

Status: {'✓ PASS' if failed == 0 else '✗ FAIL'}

{'='*80}

Detailed Results:
"""
        
        # Group by category
        categories = {}
        for result in self.results:
            category = result.name.split(':')[0]
            if category not in categories:
                categories[category] = []
            categories[category].append(result)
        
        for category, results in sorted(categories.items()):
            cat_passed = sum(1 for r in results if r.status == 'pass')
            cat_total = len(results)
            cat_rate = (cat_passed / cat_total * 100) if cat_total > 0 else 0
            
            report += f"\n{category.upper()}: {cat_passed}/{cat_total} ({cat_rate:.1f}%)\n"
            report += f"{'-'*80}\n"
            
            for result in results:
                status_symbol = '✓' if result.status == 'pass' else '✗' if result.status == 'fail' else '⊘'
                report += f"  {status_symbol} {result.name}\n"
                report += f"     {result.message}\n"
                if result.status == 'fail':
                    report += f"     Duration: {result.duration:.2f}s\n"
        
        report += f"\n{'='*80}\n"
        
        # Failed tests summary
        if failed > 0:
            report += "\nFailed Tests:\n"
            report += f"{'-'*80}\n"
            for result in self.results:
                if result.status == 'fail':
                    report += f"  ✗ {result.name}\n"
                    report += f"     {result.message}\n\n"
        
        return report
    
    def save_report(self, filename: str):
        """Save report to file"""
        report = self.generate_report()
        report_file = self.output_dir / filename
        
        with open(report_file, 'w') as f:
            f.write(report)
        
        print(report)
        print(f"\nReport saved to: {report_file}")
        
        # Also save JSON version
        json_file = self.output_dir / f"{filename}.json"
        with open(json_file, 'w') as f:
            json.dump([{
                'name': r.name,
                'status': r.status,
                'message': r.message,
                'duration': r.duration,
                'file_size': r.file_size
            } for r in self.results], f, indent=2)
        
        print(f"JSON report saved to: {json_file}")
    
    def cleanup(self):
        """Cleanup resources"""
        if hasattr(self, 'driver'):
            self.driver.quit()
            print("\n✓ WebDriver closed")

def main():
    # Configuration
    arclang_bin = './target/release/arclang'  # Use release build for speed
    examples_dir = '/Users/malek/Arclang/examples'
    output_dir = '/Users/malek/arclang/test_results'
    
    # Check if binary exists
    if not os.path.exists(arclang_bin):
        print(f"✗ ArcLang binary not found at {arclang_bin}")
        print("  Please build first: cargo build --release --bin arclang")
        sys.exit(1)
    
    # Run tests
    tester = ArcLangTester(arclang_bin, examples_dir, output_dir)
    tester.run_all_tests()
    tester.save_report('test_report.txt')
    
    # Exit with appropriate code
    failed = sum(1 for r in tester.results if r.status == 'fail')
    sys.exit(0 if failed == 0 else 1)

if __name__ == '__main__':
    main()
