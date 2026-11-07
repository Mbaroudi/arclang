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
if isfile('data/LA/LA_StateMachine_7.sldd')
	LA_StateMachine_7 = Simulink.data.dictionary.open('data/LA/LA_StateMachine_7.sldd');
else
	LA_StateMachine_7 = Simulink.data.dictionary.create('data/LA/LA_StateMachine_7.sldd');
end
addDataSource(LA_StateMachine_7, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_StateMachine_7);
clear LA_StateMachine_7;
file = proj.addFile('data/LA/LA_StateMachine_7.sldd');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_StateMachine_7');
set_param('LA_StateMachine_7', 'Tag', 'ModelerId:42711d17-107f-44e3-b968-0c040842f925');
set_param('LA_StateMachine_7', 'DataDictionary', 'LA_StateMachine_7.sldd');
states = containers.Map;
add_block('sflib/Chart', 'LA_StateMachine_7/StateMachine_7');
set_param('LA_StateMachine_7/StateMachine_7', 'Tag', 'ModelerId:42711d17-107f-44e3-b968-0c040842f925');
rt = sfroot;
ch = rt.find('-isa','Stateflow.Chart','Path', 'LA_StateMachine_7/StateMachine_7');
parent = ch;
state = Stateflow.State(parent);
state.Position = [30 30 52 67];
state.Tag = 'ModelerId:905417ea-1182-40f4-bcef-1dc96e171b16';
state.labelString = 'State_1';
state.Decomposition = 'EXCLUSIVE_OR';
states('905417ea-1182-40f4-bcef-1dc96e171b16') = state;
parent = ch;
state = Stateflow.State(parent);
state.Position = [550 200 52 67];
state.Tag = 'ModelerId:95762d74-8b87-4946-9af6-e162663e9c8d';
state.labelString = 'State_2';
state.Decomposition = 'EXCLUSIVE_OR';
states('95762d74-8b87-4946-9af6-e162663e9c8d') = state;
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:691b72a3-f906-4e6b-8017-405664ac3299';
source = states('905417ea-1182-40f4-bcef-1dc96e171b16');
transition.source = source;
transition.SourceOClock = 3.8855017059025996;
destination = states('95762d74-8b87-4946-9af6-e162663e9c8d');
transition.Destination = destination;
transition.DestinationOClock = 9.36285090182196;
transition.LabelString = [ '...Guard:[pipo !]'];
clear states rt ch parent transition destination junction myVar source state;
try
	Simulink.BlockDiagram.arrangeSystem('LA_StateMachine_7');
catch
end
save_system('LA_StateMachine_7', fullfile(project_path, 'models/LA/LA_StateMachine_7.slx'));
file = proj.addFile('models/LA/LA_StateMachine_7.slx');
addLabel(file, 'Capella Layer', 'LA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
