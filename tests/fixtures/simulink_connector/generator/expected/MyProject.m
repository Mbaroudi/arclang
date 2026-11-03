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
new_system('MyModel');
states = containers.Map;
add_block('sflib/Chart', 'MyModel/MyChart');
set_param('MyModel/MyChart', 'Position', '[200 20 250 70]')
rt = sfroot;
ch = rt.find('-isa','Stateflow.Chart','Path', 'MyModel/MyChart');
myVar = Stateflow.Data(ch);
myVar.Name = 'myInput2';
myVar.Scope = 'Input';
myVar.Port = 2;
myVar = Stateflow.Data(ch);
myVar.Name = 'out1';
myVar.Scope = 'Output';
myVar.Port = 1;
myVar = Stateflow.Data(ch);
myVar.Name = 'myInput1';
myVar.Scope = 'Input';
myVar.Port = 1;
handles = get_param('MyModel/MyChart', 'porthandles');
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
add_block('simulink/Sources/Constant', 'MyModel/MyCst1');
set_param('MyModel/MyCst1', 'Position', '[10 10 40 40]')
set_param('MyModel/MyCst1', 'value', '2')
handles = get_param('MyModel/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyModel/MyCst2');
set_param('MyModel/MyCst2', 'Position', '[10 100 40 130]')
set_param('MyModel/MyCst2', 'value', '5')
handles = get_param('MyModel/MyCst2', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'MyModel/Display');
set_param('MyModel/Display', 'Position', '[350 20 400 40]')
handles = get_param('MyModel/MyCst1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel/MyChart', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel', source, target, 'autorouting','smart');
set_param(line, 'Name', '1____chart');
clear line handles source target;
handles = get_param('MyModel/MyCst2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel/MyChart', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyModel', source, target, 'autorouting','smart');
set_param(line, 'Name', '2____chart');
clear line handles source target;
handles = get_param('MyModel/MyChart', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyModel/Display', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyModel', source, target, 'autorouting','smart');
set_param(line, 'Name', 'chart____display');
clear line handles source target;
save_system('MyModel', fullfile(project_path, 'models/MyModel.slx'));
file = proj.addFile('models/MyModel.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
