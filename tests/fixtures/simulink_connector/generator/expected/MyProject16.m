if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject16');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject16', 'Folder', project_path);
end
cd(project_path);
new_system('MyModel');
add_block('simulink/Ports & Subsystems/In1', 'MyModel/in');
set_param('MyModel/in', 'Port', '1');
add_block('simulink/Ports & Subsystems/Out1', 'MyModel/out');
set_param('MyModel/out', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'MyModel/in2');
set_param('MyModel/in2', 'Port', '2');
add_block('simulink/Ports & Subsystems/Out1', 'MyModel/out2');
set_param('MyModel/out2', 'Port', '2');
handles = get_param('MyModel/in', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel/out', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel', source, target, 'autorouting','smart');
clear line handles source target;
t_Block = add_block('simulink/Sinks/Terminator', 'MyModel/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('MyModel/in2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('MyModel', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'MyModel/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('MyModel/out2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('MyModel', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('MyModel');
catch
end
save_system('MyModel', fullfile(project_path, 'models/MyModel.slx'));
file = proj.addFile('models/MyModel.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
