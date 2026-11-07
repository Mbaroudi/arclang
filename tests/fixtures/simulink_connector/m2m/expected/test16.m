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
if isfile('data/LA/LA_Copy_of_StateMachine_1.sldd')
	LA_Copy_of_StateMachine_1 = Simulink.data.dictionary.open('data/LA/LA_Copy_of_StateMachine_1.sldd');
else
	LA_Copy_of_StateMachine_1 = Simulink.data.dictionary.create('data/LA/LA_Copy_of_StateMachine_1.sldd');
end
addDataSource(LA_Copy_of_StateMachine_1, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_Copy_of_StateMachine_1);
clear LA_Copy_of_StateMachine_1;
file = proj.addFile('data/LA/LA_Copy_of_StateMachine_1.sldd');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_Copy_of_StateMachine_1');
set_param('LA_Copy_of_StateMachine_1', 'Tag', 'ModelerId:df7272af-d901-46c1-81dc-8e1e17e7212e');
set_param('LA_Copy_of_StateMachine_1', 'DataDictionary', 'LA_Copy_of_StateMachine_1.sldd');
states = containers.Map;
add_block('sflib/Chart', 'LA_Copy_of_StateMachine_1/Copy_of_StateMachine_1');
set_param('LA_Copy_of_StateMachine_1/Copy_of_StateMachine_1', 'Tag', 'ModelerId:df7272af-d901-46c1-81dc-8e1e17e7212e');
rt = sfroot;
ch = rt.find('-isa','Stateflow.Chart','Path', 'LA_Copy_of_StateMachine_1/Copy_of_StateMachine_1');
parent = ch;
state = Stateflow.State(parent);
state.Position = [210 360 82 67];
state.Tag = 'ModelerId:260bf273-6c8f-44d2-8325-d16813f2b9f8';
state.labelString = 'Mode_1_copy';
state.Decomposition = 'EXCLUSIVE_OR';
states('260bf273-6c8f-44d2-8325-d16813f2b9f8') = state;
parent = ch;
state = Stateflow.State(parent);
state.Position = [30 30 82 67];
state.Tag = 'ModelerId:ac5c43dc-86e1-4722-8a3c-993cf27c7e18';
state.labelString = 'Mode_2_copy';
state.Decomposition = 'EXCLUSIVE_OR';
states('ac5c43dc-86e1-4722-8a3c-993cf27c7e18') = state;
parent = ch;
state = Stateflow.State(parent);
state.Position = [470 120 82 67];
state.Tag = 'ModelerId:8cf4c493-f2fd-44d5-ac3e-ac49835802b9';
state.labelString = 'Mode_3_copy';
state.Decomposition = 'EXCLUSIVE_OR';
states('8cf4c493-f2fd-44d5-ac3e-ac49835802b9') = state;
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:c8ed2873-42bc-4c55-9c7f-fb366ea709c9';
source = states('260bf273-6c8f-44d2-8325-d16813f2b9f8');
transition.source = source;
transition.SourceOClock = 11.942143180369053;
destination = states('ac5c43dc-86e1-4722-8a3c-993cf27c7e18');
transition.Destination = destination;
transition.DestinationOClock = 6.059663686941535;
transition.LabelString = ['at(FunctionalExchange 1 is true)'];
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:8e2c3452-c837-4240-829b-20fca209a074';
source = states('260bf273-6c8f-44d2-8325-d16813f2b9f8');
transition.source = source;
transition.SourceOClock = 11.942143180369053;
destination = states('8cf4c493-f2fd-44d5-ac3e-ac49835802b9');
transition.Destination = destination;
transition.DestinationOClock = 6.059663686941535;
transition.LabelString = [ '...Guard:[myCondition]'];
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:0f44203c-4ace-47f0-8b99-c79dc691b680';
source = states('ac5c43dc-86e1-4722-8a3c-993cf27c7e18');
transition.source = source;
transition.SourceOClock = 3.570090965635079;
destination = states('8cf4c493-f2fd-44d5-ac3e-ac49835802b9');
transition.Destination = destination;
transition.DestinationOClock = 8.456204919124648;
clear states rt ch parent transition destination junction myVar source state;
try
	Simulink.BlockDiagram.arrangeSystem('LA_Copy_of_StateMachine_1');
catch
end
save_system('LA_Copy_of_StateMachine_1', fullfile(project_path, 'models/LA/LA_Copy_of_StateMachine_1.slx'));
file = proj.addFile('models/LA/LA_Copy_of_StateMachine_1.slx');
addLabel(file, 'Capella Layer', 'LA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
