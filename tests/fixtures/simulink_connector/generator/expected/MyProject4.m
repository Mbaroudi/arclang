if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject4');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject4', 'Folder', project_path);
end
cd(project_path);
new_system('MyModel4');
add_block('simulink/Sources/Constant', 'MyModel4/MyCst1');
set_param('MyModel4/MyCst1', 'Position', '[10 10 40 40]')
set_param('MyModel4/MyCst1', 'value', '2')
handles = get_param('MyModel4/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyModel4/MyCst2');
set_param('MyModel4/MyCst2', 'Position', '[10 100 40 130]')
set_param('MyModel4/MyCst2', 'value', '5')
handles = get_param('MyModel4/MyCst2', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Creator', 'MyModel4/creator');
set_param('MyModel4/creator', 'Position', '[100 10 110 60]')
handles = get_param('MyModel4/creator', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'MyModel4/Display');
set_param('MyModel4/Display', 'Position', '[350 20 390 60]')
add_block('simulink/Ports & Subsystems/Subsystem', 'MyModel4/MySystem');
set_param('MyModel4/MySystem', 'Position', '[200 20 280 100]')
CapSLBridge.internal.cleanSubsystem('MyModel4/MySystem');
add_block('simulink/Ports & Subsystems/In1', 'MyModel4/MySystem/in1');
set_param('MyModel4/MySystem/in1', 'Port', '1');
set_param('MyModel4/MySystem/in1', 'Position', '[10 100 40 120]');
add_block('simulink/Ports & Subsystems/Out1', 'MyModel4/MySystem/out1');
set_param('MyModel4/MySystem/out1', 'Port', '1');
set_param('MyModel4/MySystem/out1', 'Position', '[200 50 230 70]');
add_block('simulink/Sources/Constant', 'MyModel4/MySystem/cst');
set_param('MyModel4/MySystem/cst', 'Position', '[10 10 40 40]')
set_param('MyModel4/MySystem/cst', 'value', '10')
handles = get_param('MyModel4/MySystem/cst', 'porthandles');
clear handles;
add_block('simulink/Math Operations/Product', 'MyModel4/MySystem/product');
set_param('MyModel4/MySystem/product', 'Position', '[100 60 130 90]')
handles = get_param('MyModel4/MySystem/product', 'porthandles');
clear handles;
handles = get_param('MyModel4/MySystem/cst', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel4/MySystem/product', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel4/MySystem', source, target, 'autorouting','smart');
set_param(line, 'Name', 'cst____product');
clear line handles source target;
handles = get_param('MyModel4/MySystem/in1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel4/MySystem/product', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyModel4/MySystem', source, target, 'autorouting','smart');
set_param(line, 'Name', 'in____product');
clear line handles source target;
handles = get_param('MyModel4/MySystem/product', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel4/MySystem/out1', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel4/MySystem', source, target, 'autorouting','smart');
set_param(line, 'Name', 'product____out');
clear line handles source target;
handles = get_param('MyModel4/MyCst1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel4/creator', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyModel4', source, target, 'autorouting','smart');
set_param(line, 'Name', '1____creator');
clear line handles source target;
handles = get_param('MyModel4/MyCst2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel4/creator', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel4', source, target, 'autorouting','smart');
set_param(line, 'Name', '2____creator');
clear line handles source target;
handles = get_param('MyModel4/creator', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel4/MySystem', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'creator____sys');
clear line handles source target;
handles = get_param('MyModel4/MySystem', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel4/Display', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sys____display');
clear line handles source target;
save_system('MyModel4', fullfile(project_path, 'models/MyModel4.slx'));
file = proj.addFile('models/MyModel4.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
