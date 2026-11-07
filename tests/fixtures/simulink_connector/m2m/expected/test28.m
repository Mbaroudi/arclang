if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'mystate');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'mystate', 'Folder', project_path);
end
cd(project_path);
if isfile('data/LA/LA_FunctionalExchanges_Categories.sldd')
	LA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/LA/LA_FunctionalExchanges_Categories.sldd');
else
	LA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/LA/LA_FunctionalExchanges_Categories.sldd');
end
saveChanges(LA_FunctionalExchanges_Categories);
clear LA_FunctionalExchanges_Categories;
file = proj.addFile('data/LA/LA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_StateMachine_10.sldd')
	LA_StateMachine_10 = Simulink.data.dictionary.open('data/LA/LA_StateMachine_10.sldd');
else
	LA_StateMachine_10 = Simulink.data.dictionary.create('data/LA/LA_StateMachine_10.sldd');
end
addDataSource(LA_StateMachine_10, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_StateMachine_10);
clear LA_StateMachine_10;
file = proj.addFile('data/LA/LA_StateMachine_10.sldd');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_StateMachine_10');
set_param('LA_StateMachine_10', 'Tag', 'ModelerId:8637f179-ef09-47c6-96bb-4c338bd13845');
set_param('LA_StateMachine_10', 'DataDictionary', 'LA_StateMachine_10.sldd');
states = containers.Map;
add_block('sflib/Chart', 'LA_StateMachine_10/StateMachine_10');
set_param('LA_StateMachine_10/StateMachine_10', 'Tag', 'ModelerId:8637f179-ef09-47c6-96bb-4c338bd13845');
rt = sfroot;
ch = rt.find('-isa','Stateflow.Chart','Path', 'LA_StateMachine_10/StateMachine_10');
parent = ch;
state = Stateflow.State(parent);
state.Position = [30 30 80 129];
state.Tag = 'ModelerId:57eeb146-68df-42ad-b3d3-2ad46fac9233';
state.labelString = 'State_2';
state.Decomposition = 'EXCLUSIVE_OR';
states('57eeb146-68df-42ad-b3d3-2ad46fac9233') = state;
junction = Stateflow.Junction(ch);
junction.type = 'CONNECTIVE';
junction.Tag = 'ModelerId:2e39ba06-57a1-4469-a9e2-20cce248c5fe';
junction.Position.Center = [58 88];
junction.Position.Radius = 8;
states('2e39ba06-57a1-4469-a9e2-20cce248c5fe') = junction;
clear states rt ch parent transition destination junction myVar source state;
try
	Simulink.BlockDiagram.arrangeSystem('LA_StateMachine_10');
catch
end
save_system('LA_StateMachine_10', fullfile(project_path, 'models/LA/LA_StateMachine_10.slx'));
file = proj.addFile('models/LA/LA_StateMachine_10.slx');
addLabel(file, 'Capella Layer', 'LA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;