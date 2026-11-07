if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject2');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject2', 'Folder', project_path);
end
cd(project_path);
new_system('MyModel2');
add_block('simulink/Sources/Constant', 'MyModel2/MyCst1');
set_param('MyModel2/MyCst1', 'value', '2')
handles = get_param('MyModel2/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyModel2/MyCst2');
set_param('MyModel2/MyCst2', 'value', '5')
handles = get_param('MyModel2/MyCst2', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'MyModel2/Display');
set_param('MyModel2/Display', 'Description', 'My block description');
add_block('simulink/Ports & Subsystems/Subsystem', 'MyModel2/MySystem');
set_param('MyModel2/MySystem', 'Description', 'MySystem description');
CapSLBridge.internal.cleanSubsystem('MyModel2/MySystem');
add_block('simulink/Ports & Subsystems/In1', 'MyModel2/MySystem/in1');
set_param('MyModel2/MySystem/in1', 'Port', '1');
add_block('simulink/Ports & Subsystems/Out1', 'MyModel2/MySystem/out1');
set_param('MyModel2/MySystem/out1', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'MyModel2/MySystem/in2');
set_param('MyModel2/MySystem/in2', 'Port', '2');
add_block('simulink/Ports & Subsystems/In1', 'MyModel2/MySystem/in3');
set_param('MyModel2/MySystem/in3', 'Port', '3');
set_param('MyModel2/MySystem/in3', 'Description', 'My port description');
t_Block = add_block('simulink/Sinks/Terminator', 'MyModel2/MySystem/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('MyModel2/MySystem/in3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('MyModel2/MySystem', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'MyModel2/MySystem/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('MyModel2/MySystem/in2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('MyModel2/MySystem', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'MyModel2/MySystem/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('MyModel2/MySystem/in1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('MyModel2/MySystem', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'MyModel2/MySystem/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('MyModel2/MySystem/out1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('MyModel2/MySystem', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('MyModel2/MySystem');
catch
end
handles = get_param('MyModel2/MyCst1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel2/MySystem', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyModel2', source, target, 'autorouting','smart');
set_param(line, 'Name', '1____sys');
set_param(line, 'Description', 'My line description');
clear line handles source target;
handles = get_param('MyModel2/MyCst2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel2/MySystem', 'PortHandles');
target = handles.Inport(3);
line = add_line('MyModel2', source, target, 'autorouting','smart');
set_param(line, 'Name', '2____sys');
clear line handles source target;
handles = get_param('MyModel2/MySystem', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel2/Display', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sys____display');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('MyModel2');
catch
end
save_system('MyModel2', fullfile(project_path, 'models/MyModel2.slx'));
file = proj.addFile('models/MyModel2.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
