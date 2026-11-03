if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject17');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject17', 'Folder', project_path);
end
cd(project_path);
new_system('otherSystem');
add_block('simulink/Ports & Subsystems/In1', 'otherSystem/in1');
set_param('otherSystem/in1', 'Port', '1');
t_Block = add_block('simulink/Sinks/Terminator', 'otherSystem/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('otherSystem/in1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('otherSystem', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('otherSystem');
catch
end
save_system('otherSystem', fullfile(project_path, 'models/otherSystem.slx'));
file = proj.addFile('models/otherSystem.slx');
new_system('mainSystem');
add_block('simulink/Ports & Subsystems/In1', 'mainSystem/in1');
set_param('mainSystem/in1', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'mainSystem/in2');
set_param('mainSystem/in2', 'Port', '2');
add_block('simulink/Ports & Subsystems/Model', 'mainSystem/other1');
set_param('mainSystem/other1', 'ModelFile', 'otherSystem')
add_block('simulink/Ports & Subsystems/Model', 'mainSystem/other2');
set_param('mainSystem/other2', 'ModelFile', 'otherSystem')
handles = get_param('mainSystem/in2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('mainSystem/other2', 'PortHandles');
target = handles.Inport(1);
line = add_line('mainSystem', source, target, 'autorouting','smart');
set_param(line, 'Name', 'line2');
clear line handles source target;
handles = get_param('mainSystem/in1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('mainSystem/other1', 'PortHandles');
target = handles.Inport(1);
line = add_line('mainSystem', source, target, 'autorouting','smart');
set_param(line, 'Name', 'line1');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('mainSystem');
catch
end
save_system('mainSystem', fullfile(project_path, 'models/mainSystem.slx'));
file = proj.addFile('models/mainSystem.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
