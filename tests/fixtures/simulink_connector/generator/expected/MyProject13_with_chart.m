if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject13');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject13', 'Folder', project_path);
end
cd(project_path);
new_system('MySystem13');
states = containers.Map;
add_block('sflib/Chart', 'MySystem13/MyChart');
set_param('MySystem13/MyChart', 'Position', '[200 20 250 70]')
rt = sfroot;
ch = rt.find('-isa','Stateflow.Chart','Path', 'MySystem13/MyChart');
myVar = Stateflow.Data(ch);
myVar.Name = 'myInput2';
myVar.Scope = 'Input';
myVar.Port = 2;
myVar.DataType = 'double';
myVar = Stateflow.Data(ch);
myVar.Name = 'out1';
myVar.Scope = 'Output';
myVar.Port = 1;
myVar.DataType = 'double';
myVar = Stateflow.Data(ch);
myVar.Name = 'myInput1';
myVar.Scope = 'Input';
myVar.Port = 1;
myVar.DataType = 'double';
handles = get_param('MySystem13/MyChart', 'porthandles');
clear handles;
parent = ch;
state = Stateflow.State(parent);
state.Position = [60 60 60 60];
state.Tag = 'ModelerId:264be410-bcec-42ac-a1cf-fed82ee7e716';
state.labelString = ['aState', 10, 'en: out1 = myInput2 + myInput1'];
state.Decomposition = 'EXCLUSIVE_OR';
states('264be410-bcec-42ac-a1cf-fed82ee7e716') = state;
transition = Stateflow.Transition(ch);
transition.SourceOClock = 0;
destination = states('264be410-bcec-42ac-a1cf-fed82ee7e716');
transition.Destination = destination;
transition.DestinationOClock = 0;
clear states rt ch parent transition destination junction myVar source state;
add_block('simulink/Sources/Constant', 'MySystem13/MyCst1');
set_param('MySystem13/MyCst1', 'Position', '[10 10 40 40]')
set_param('MySystem13/MyCst1', 'value', '2')
handles = get_param('MySystem13/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MySystem13/MyCst2');
set_param('MySystem13/MyCst2', 'Position', '[10 100 40 130]')
set_param('MySystem13/MyCst2', 'OutDataTypeStr', 'double');
set_param('MySystem13/MyCst2', 'value', '5')
handles = get_param('MySystem13/MyCst2', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'MySystem13/Display');
set_param('MySystem13/Display', 'Position', '[350 20 400 40]')
handles = get_param('MySystem13/MyCst1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem13/MyChart', 'PortHandles');
target = handles.Inport(1);
line = add_line('MySystem13', source, target, 'autorouting','smart');
set_param(line, 'Name', '1____chart');
clear line handles source target;
handles = get_param('MySystem13/MyCst2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem13/MyChart', 'PortHandles');
target = handles.Inport(2);
line = add_line('MySystem13', source, target, 'autorouting','smart');
set_param(line, 'Name', '2____chart');
clear line handles source target;
handles = get_param('MySystem13/MyChart', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem13/Display', 'PortHandles');
target = handles.Inport(1);
line = add_line('MySystem13', source, target, 'autorouting','smart');
set_param(line, 'Name', 'chart____display');
clear line handles source target;
save_system('MySystem13', fullfile(project_path, 'models/MySystem13.slx'));
file = proj.addFile('models/MySystem13.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
