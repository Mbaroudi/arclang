if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject9');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject9', 'Folder', project_path);
end
cd(project_path);
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
new_system('MyModel9');
add_block('simulink/Ports & Subsystems/Model', 'MyModel9/MyRefModel');
set_param('MyModel9/MyRefModel', 'Position', '[10 10 260 60]')
set_param('MyModel9/MyRefModel', 'ModelFile', 'MyModel5')
save_system('MyModel9', fullfile(project_path, 'models/MyModel9.slx'));
file = proj.addFile('models/MyModel9.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
