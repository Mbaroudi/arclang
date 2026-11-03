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
if isfile('data/LA/LA_StateMachine_11.sldd')
	LA_StateMachine_11 = Simulink.data.dictionary.open('data/LA/LA_StateMachine_11.sldd');
else
	LA_StateMachine_11 = Simulink.data.dictionary.create('data/LA/LA_StateMachine_11.sldd');
end
addDataSource(LA_StateMachine_11, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_StateMachine_11);
clear LA_StateMachine_11;
file = proj.addFile('data/LA/LA_StateMachine_11.sldd');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_StateMachine_11');
set_param('LA_StateMachine_11', 'Tag', 'ModelerId:faa4dfe6-5a3d-4d76-ac44-c3607d19c376');
set_param('LA_StateMachine_11', 'DataDictionary', 'LA_StateMachine_11.sldd');
states = containers.Map;
add_block('sflib/Chart', 'LA_StateMachine_11/StateMachine_11');
set_param('LA_StateMachine_11/StateMachine_11', 'Tag', 'ModelerId:faa4dfe6-5a3d-4d76-ac44-c3607d19c376');
rt = sfroot;
ch = rt.find('-isa','Stateflow.Chart','Path', 'LA_StateMachine_11/StateMachine_11');
junction = Stateflow.Junction(ch);
junction.type = 'CONNECTIVE';
junction.Tag = 'ModelerId:da3ac45e-60d9-46be-aad7-1121f0e09547';
junction.Position.Center = [38 38];
junction.Position.Radius = 8;
states('da3ac45e-60d9-46be-aad7-1121f0e09547') = junction;
clear states rt ch parent transition destination junction myVar source state;
try
	Simulink.BlockDiagram.arrangeSystem('LA_StateMachine_11');
catch
end
save_system('LA_StateMachine_11', fullfile(project_path, 'models/LA/LA_StateMachine_11.slx'));
file = proj.addFile('models/LA/LA_StateMachine_11.slx');
addLabel(file, 'Capella Layer', 'LA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;