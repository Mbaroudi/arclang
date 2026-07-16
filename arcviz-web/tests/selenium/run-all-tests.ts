import { TestReporter } from './config.js';
import { runAuthTests } from './01-auth.test.js';
import { runEditorTests } from './02-editor.test.js';
import { runVisualizerTests } from './03-visualizer.test.js';
import { runChatTests } from './04-chat.test.js';
import { run7DVisualizerTests } from './05-7d-visualizer.test.js';
import { runMBSECapellaTests } from './06-mbse-capella.test.js';
import { runMetamodelCompleteTests } from './07-metamodel-complete.test.js';

async function main() {
  console.log('\n' + '='.repeat(80));
  console.log('  ARCLANG PLATFORM - COMPLETE SELENIUM TEST SUITE');
  console.log('  Including MBSE Capella Feature Validation');
  console.log('='.repeat(80) + '\n');

  const reporter = new TestReporter();

  console.log('\n📋 Test Suite 1: Authentication Tests');
  console.log('─'.repeat(80));
  await runAuthTests(reporter);

  console.log('\n\n📋 Test Suite 2: Editor Tests');
  console.log('─'.repeat(80));
  await runEditorTests(reporter);

  console.log('\n\n📋 Test Suite 3: Visualizer Tests');
  console.log('─'.repeat(80));
  await runVisualizerTests(reporter);

  console.log('\n\n📋 Test Suite 4: Chat AI Tests');
  console.log('─'.repeat(80));
  await runChatTests(reporter);

  console.log('\n\n📋 Test Suite 5: 7D Visualizer Tests');
  console.log('─'.repeat(80));
  await run7DVisualizerTests(reporter);

  console.log('\n\n📋 Test Suite 6: MBSE Capella Features (15 Features)');
  console.log('─'.repeat(80));
  await runMBSECapellaTests(reporter);

  console.log('\n\n📋 Test Suite 7: Complete Metamodel Elements');
  console.log('─'.repeat(80));
  await runMetamodelCompleteTests(reporter);

  console.log('\n\n' + '='.repeat(80));
  console.log(reporter.generateReport());

  const reportPath = reporter.saveReport(`complete-test-report-${Date.now()}.txt`);
  console.log(`\n📄 Full report saved to: ${reportPath}\n`);

  process.exit(0);
}

main().catch(error => {
  console.error('\n❌ Test suite failed:', error);
  process.exit(1);
});
