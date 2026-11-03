if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject6');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject6', 'Folder', project_path);
end
cd(project_path);
new_system('MyProject');
add_block('simulink/Ports & Subsystems/Subsystem', 'MyProject/System_A');
set_param('MyProject/System_A', 'Position', '[300 20 380 100]')
CapSLBridge.internal.cleanSubsystem('MyProject/System_A');
add_block('simulink/Ports & Subsystems/In1', 'MyProject/System_A/in1');
set_param('MyProject/System_A/in1', 'Port', '1');
set_param('MyProject/System_A/in1', 'Position', '[10 100 40 120]');
add_block('simulink/Ports & Subsystems/Subsystem', 'MyProject/System_B');
set_param('MyProject/System_B', 'Position', '[40 20 120 100]')
CapSLBridge.internal.cleanSubsystem('MyProject/System_B');
add_block('simulink/Ports & Subsystems/Out1', 'MyProject/System_B/out1');
set_param('MyProject/System_B/out1', 'Port', '1');
set_param('MyProject/System_B/out1', 'Position', '[1 1 2 2]');
handles = get_param('MyProject/System_B', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_A', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyProject', source, target, 'autorouting','smart');
clear line handles source target;
save_system('MyProject', fullfile(project_path, 'models/MyProject.slx'));
file = proj.addFile('models/MyProject.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
