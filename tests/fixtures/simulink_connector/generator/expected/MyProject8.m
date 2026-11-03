if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject', 'Folder', project_path);
end
cd(project_path);
new_system('MySystem8');
add_block('simulink/Ports & Subsystems/In1', 'MySystem8/in2');
set_param('MySystem8/in2', 'Port', '1');
set_param('MySystem8/in2', 'Position', '[10 10 40 30]');
add_block('simulink/Ports & Subsystems/In1', 'MySystem8/in1');
set_param('MySystem8/in1', 'Port', '1');
set_param('MySystem8/in1', 'Position', '[10 60 40 80]');
add_block('simulink/Ports & Subsystems/Out1', 'MySystem8/out1');
set_param('MySystem8/out1', 'Port', '1');
set_param('MySystem8/out1', 'Position', '[460 250 490 270]');
add_block('simulink/Ports & Subsystems/Subsystem', 'MySystem8/System_A');
set_param('MySystem8/System_A', 'Position', '[100 20 180 100]')
CapSLBridge.internal.cleanSubsystem('MySystem8/System_A');
add_block('simulink/Ports & Subsystems/In1', 'MySystem8/System_A/in1');
set_param('MySystem8/System_A/in1', 'Port', '1');
set_param('MySystem8/System_A/in1', 'Position', '[10 100 40 120]');
add_block('simulink/Ports & Subsystems/Out1', 'MySystem8/System_A/out');
set_param('MySystem8/System_A/out', 'Port', '1');
set_param('MySystem8/System_A/out', 'Position', '[200 10 230 30]');
add_block('simulink/Ports & Subsystems/In1', 'MySystem8/System_A/in2');
set_param('MySystem8/System_A/in2', 'Port', '2');
set_param('MySystem8/System_A/in2', 'Position', '[10 10 40 30]');
add_block('simulink/Ports & Subsystems/Subsystem', 'MySystem8/System_B');
set_param('MySystem8/System_B', 'Position', '[300 200 380 280]')
CapSLBridge.internal.cleanSubsystem('MySystem8/System_B');
add_block('simulink/Ports & Subsystems/Out1', 'MySystem8/System_B/out1');
set_param('MySystem8/System_B/out1', 'Port', '1');
set_param('MySystem8/System_B/out1', 'Position', '[10 100 40 120]');
add_block('simulink/Ports & Subsystems/In1', 'MySystem8/System_B/in');
set_param('MySystem8/System_B/in', 'Port', '1');
set_param('MySystem8/System_B/in', 'Position', '[10 10 40 30]');
handles = get_param('MySystem8/System_A', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem8/System_B', 'PortHandles');
target = handles.Inport(1);
line = add_line('MySystem8', source, target, 'autorouting','smart');
clear line handles source target;
handles = get_param('MySystem8/in1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem8/System_A', 'PortHandles');
target = handles.Inport(1);
line = add_line('MySystem8', source, target, 'autorouting','smart');
clear line handles source target;
handles = get_param('MySystem8/in2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem8/System_A', 'PortHandles');
target = handles.Inport(2);
line = add_line('MySystem8', source, target, 'autorouting','smart');
clear line handles source target;
handles = get_param('MySystem8/System_B', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem8/out1', 'PortHandles');
target = handles.Inport(1);
line = add_line('MySystem8', source, target, 'autorouting','smart');
clear line handles source target;
save_system('MySystem8', fullfile(project_path, 'models/MySystem8.slx'));
file = proj.addFile('models/MySystem8.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
