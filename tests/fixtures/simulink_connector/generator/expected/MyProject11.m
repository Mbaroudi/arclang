if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject11');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject11', 'Folder', project_path);
end
cd(project_path);
new_system('system10');
add_block('simulink/Sources/Constant', 'system10/MyCst1');
set_param('system10/MyCst1', 'Position', '[10 10 40 40]')
set_param('system10/MyCst1', 'value', '2')
handles = get_param('system10/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'system10/Display');
set_param('system10/Display', 'Position', '[350 20 390 60]')
save_system('system10', fullfile(project_path, 'models/system10.slx'));
file = proj.addFile('models/system10.slx');
new_system('MyModel5');
add_block('simulink/Sources/Constant', 'MyModel5/MyCst1');
set_param('MyModel5/MyCst1', 'Position', '[10 10 40 40]')
set_param('MyModel5/MyCst1', 'value', '2')
handles = get_param('MyModel5/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyModel5/MyCst2');
set_param('MyModel5/MyCst2', 'Position', '[10 100 40 130]')
set_param('MyModel5/MyCst2', 'value', '5')
handles = get_param('MyModel5/MyCst2', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Creator', 'MyModel5/creator');
set_param('MyModel5/creator', 'Position', '[100 10 110 60]')
handles = get_param('MyModel5/creator', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'MyModel5/Display');
set_param('MyModel5/Display', 'Position', '[350 20 390 60]')
add_block('simulink/Ports & Subsystems/Subsystem', 'MyModel5/MySystem1');
set_param('MyModel5/MySystem1', 'Position', '[200 20 280 100]')
CapSLBridge.internal.cleanSubsystem('MyModel5/MySystem1');
add_block('simulink/Ports & Subsystems/In1', 'MyModel5/MySystem1/in1');
set_param('MyModel5/MySystem1/in1', 'Port', '1');
set_param('MyModel5/MySystem1/in1', 'Position', '[10 50 40 70]');
add_block('simulink/Ports & Subsystems/Out1', 'MyModel5/MySystem1/out1');
set_param('MyModel5/MySystem1/out1', 'Port', '1');
set_param('MyModel5/MySystem1/out1', 'Position', '[300 50 330 70]');
add_block('simulink/Ports & Subsystems/Subsystem', 'MyModel5/MySystem1/MySystem2');
set_param('MyModel5/MySystem1/MySystem2', 'Position', '[150 20 230 100]')
CapSLBridge.internal.cleanSubsystem('MyModel5/MySystem1/MySystem2');
add_block('simulink/Ports & Subsystems/In1', 'MyModel5/MySystem1/MySystem2/in1');
set_param('MyModel5/MySystem1/MySystem2/in1', 'Port', '1');
set_param('MyModel5/MySystem1/MySystem2/in1', 'Position', '[10 100 40 120]');
add_block('simulink/Ports & Subsystems/Out1', 'MyModel5/MySystem1/MySystem2/out1');
set_param('MyModel5/MySystem1/MySystem2/out1', 'Port', '1');
set_param('MyModel5/MySystem1/MySystem2/out1', 'Position', '[200 50 230 70]');
add_block('simulink/Sources/Constant', 'MyModel5/MySystem1/MySystem2/cst');
set_param('MyModel5/MySystem1/MySystem2/cst', 'Position', '[10 10 40 40]')
set_param('MyModel5/MySystem1/MySystem2/cst', 'value', '10')
handles = get_param('MyModel5/MySystem1/MySystem2/cst', 'porthandles');
clear handles;
add_block('simulink/Math Operations/Product', 'MyModel5/MySystem1/MySystem2/product');
set_param('MyModel5/MySystem1/MySystem2/product', 'Position', '[100 60 130 90]')
handles = get_param('MyModel5/MySystem1/MySystem2/product', 'porthandles');
clear handles;
handles = get_param('MyModel5/MySystem1/MySystem2/cst', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/MySystem1/MySystem2/product', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel5/MySystem1/MySystem2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'cst____product');
clear line handles source target;
handles = get_param('MyModel5/MySystem1/MySystem2/in1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/MySystem1/MySystem2/product', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyModel5/MySystem1/MySystem2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'in____product');
clear line handles source target;
handles = get_param('MyModel5/MySystem1/MySystem2/product', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/MySystem1/MySystem2/out1', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel5/MySystem1/MySystem2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'product____out');
clear line handles source target;
handles = get_param('MyModel5/MySystem1/in1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/MySystem1/MySystem2', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel5/MySystem1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'in');
clear line handles source target;
handles = get_param('MyModel5/MySystem1/MySystem2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/MySystem1/out1', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel5/MySystem1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'out');
clear line handles source target;
handles = get_param('MyModel5/MyCst1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/creator', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyModel5', source, target, 'autorouting','smart');
set_param(line, 'Name', '1____creator');
clear line handles source target;
handles = get_param('MyModel5/MyCst2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/creator', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel5', source, target, 'autorouting','smart');
set_param(line, 'Name', '2____creator');
clear line handles source target;
handles = get_param('MyModel5/creator', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/MySystem1', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel5', source, target, 'autorouting','smart');
set_param(line, 'Name', 'creator____sys');
clear line handles source target;
handles = get_param('MyModel5/MySystem1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel5/Display', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel5', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sys____display');
clear line handles source target;
save_system('MyModel5', fullfile(project_path, 'models/MyModel5.slx'));
file = proj.addFile('models/MyModel5.slx');
new_system('MyModel10');
add_block('simulink/Ports & Subsystems/Model', 'MyModel10/MyRefModel');
set_param('MyModel10/MyRefModel', 'Position', '[10 10 260 60]')
set_param('MyModel10/MyRefModel', 'ModelFile', 'MyModel5')
add_block('simulink/Ports & Subsystems/Subsystem', 'MyModel10/MySubsystem1');
set_param('MyModel10/MySubsystem1', 'Position', '[10 100 260 250]')
CapSLBridge.internal.cleanSubsystem('MyModel10/MySubsystem1');
add_block('simulink/Ports & Subsystems/Model', 'MyModel10/MySubsystem1/MyRefModel2');
set_param('MyModel10/MySubsystem1/MyRefModel2', 'Position', '[10 10 90 90]')
set_param('MyModel10/MySubsystem1/MyRefModel2', 'ModelFile', 'system10')
save_system('MyModel10', fullfile(project_path, 'models/MyModel10.slx'));
file = proj.addFile('models/MyModel10.slx');
new_system('system11');
add_block('simulink/Ports & Subsystems/In1', 'system11/in1');
set_param('system11/in1', 'Port', '1');
set_param('system11/in1', 'Position', '[20 10 50 30]');
add_block('simulink/Ports & Subsystems/Out1', 'system11/out1');
set_param('system11/out1', 'Port', '1');
set_param('system11/out1', 'Position', '[200 80 230 100]');
add_block('simulink/Sources/Constant', 'system11/MyCst1');
set_param('system11/MyCst1', 'Position', '[10 80 40 110]')
set_param('system11/MyCst1', 'value', '2')
handles = get_param('system11/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'system11/Display');
set_param('system11/Display', 'Position', '[200 10 240 50]')
handles = get_param('system11/in1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('system11/Display', 'PortHandles');
target = handles.Inport(1);
line = add_line('system11', source, target, 'autorouting','smart');
set_param(line, 'Name', 'in____display');
clear line handles source target;
handles = get_param('system11/MyCst1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('system11/out1', 'PortHandles');
target = handles.Inport(1);
line = add_line('system11', source, target, 'autorouting','smart');
set_param(line, 'Name', 'cst____out');
clear line handles source target;
save_system('system11', fullfile(project_path, 'models/system11.slx'));
file = proj.addFile('models/system11.slx');
new_system('MyModel11');
add_block('simulink/Ports & Subsystems/Model', 'MyModel11/ref2model11');
set_param('MyModel11/ref2model11', 'Position', '[100 100 180 180]')
set_param('MyModel11/ref2model11', 'ModelFile', 'system11')
handles = get_param('MyModel11/ref2model11', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyModel11/TopCst');
set_param('MyModel11/TopCst', 'Position', '[10 10 40 40]')
set_param('MyModel11/TopCst', 'value', '2')
handles = get_param('MyModel11/TopCst', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'MyModel11/TopDisplay');
set_param('MyModel11/TopDisplay', 'Position', '[350 20 390 60]')
add_block('simulink/Ports & Subsystems/Model', 'MyModel11/ref2model10');
set_param('MyModel11/ref2model10', 'Position', '[100 250 180 330]')
set_param('MyModel11/ref2model10', 'ModelFile', 'MyModel10')
handles = get_param('MyModel11/TopCst', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel11/ref2model11', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel11', source, target, 'autorouting','smart');
clear line handles source target;
handles = get_param('MyModel11/ref2model11', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel11/TopDisplay', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel11', source, target, 'autorouting','smart');
clear line handles source target;
save_system('MyModel11', fullfile(project_path, 'models/MyModel11.slx'));
file = proj.addFile('models/MyModel11.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
