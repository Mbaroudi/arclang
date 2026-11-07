if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject3');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject3', 'Folder', project_path);
end
cd(project_path);
new_system('MyModel3');
add_block('simulink/Sources/Constant', 'MyModel3/MyCst1');
set_param('MyModel3/MyCst1', 'Position', '[10 10 40 40]')
set_param('MyModel3/MyCst1', 'value', '2')
handles = get_param('MyModel3/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyModel3/MyCst2');
set_param('MyModel3/MyCst2', 'Position', '[10 100 40 130]')
set_param('MyModel3/MyCst2', 'value', '5')
handles = get_param('MyModel3/MyCst2', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Creator', 'MyModel3/creator');
set_param('MyModel3/creator', 'Position', '[100 10 110 60]')
handles = get_param('MyModel3/creator', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'MyModel3/Display');
set_param('MyModel3/Display', 'Position', '[350 20 390 60]')
add_block('simulink/Ports & Subsystems/Subsystem', 'MyModel3/MySystem');
set_param('MyModel3/MySystem', 'Position', '[200 20 280 100]')
CapSLBridge.internal.cleanSubsystem('MyModel3/MySystem');
add_block('simulink/Ports & Subsystems/In1', 'MyModel3/MySystem/in1');
set_param('MyModel3/MySystem/in1', 'Port', '1');
set_param('MyModel3/MySystem/in1', 'Position', '[10 10 40 30]');
add_block('simulink/Ports & Subsystems/Out1', 'MyModel3/MySystem/out1');
set_param('MyModel3/MySystem/out1', 'Port', '1');
set_param('MyModel3/MySystem/out1', 'Position', '[100 10 130 30]');
handles = get_param('MyModel3/MySystem/in1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel3/MySystem/out1', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel3/MySystem', source, target, 'autorouting','smart');
clear line handles source target;
handles = get_param('MyModel3/MyCst1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel3/creator', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyModel3', source, target, 'autorouting','smart');
set_param(line, 'Name', '1____creator');
clear line handles source target;
handles = get_param('MyModel3/MyCst2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel3/creator', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel3', source, target, 'autorouting','smart');
set_param(line, 'Name', '2____creator');
clear line handles source target;
handles = get_param('MyModel3/creator', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel3/MySystem', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'creator____sys');
clear line handles source target;
handles = get_param('MyModel3/MySystem', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel3/Display', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sys____display');
clear line handles source target;
save_system('MyModel3', fullfile(project_path, 'models/MyModel3.slx'));
file = proj.addFile('models/MyModel3.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
