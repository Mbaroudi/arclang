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
Enumeration_2 = Simulink.data.dictionary.EnumTypeDefinition;
appendEnumeral(Enumeration_2, 'EnumerationLiteral_1', 0, '');
removeEnumeral(Enumeration_2,1);
assignin('base', 'Enumeration_2', Enumeration_2);
if isfile('data/LA/LA_DataPkg_2.sldd')
	LA_DataPkg_2 = Simulink.data.dictionary.open('data/LA/LA_DataPkg_2.sldd');
else
	LA_DataPkg_2 = Simulink.data.dictionary.create('data/LA/LA_DataPkg_2.sldd');
end
importFromBaseWorkspace(LA_DataPkg_2, 'varList', {'Enumeration_2'}, 'existingVarsAction','overwrite');
saveChanges(LA_DataPkg_2);
clear LA_DataPkg_2;
file = proj.addFile('data/LA/LA_DataPkg_2.sldd');
addLabel(file, 'Capella Layer', 'LA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Property_1';
elems(1).Description = 'ModelerId:e1a73358-788d-4ebf-ab3b-8365b7caf57b';
elems(1).DataType = 'Enum: Enumeration_2';
elems(2) = Simulink.BusElement;
elems(2).Name = 'Property_2';
elems(2).Description = 'ModelerId:58f2651c-56d0-4236-ab93-eb0fa0d038be';
elems(2).DataType = 'double';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Description = 'ModelerId:80382e73-56b4-40e3-9a41-e7822268dab2';
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
if isfile('data/LA/LA_DataPkg_1.sldd')
	LA_DataPkg_1 = Simulink.data.dictionary.open('data/LA/LA_DataPkg_1.sldd');
else
	LA_DataPkg_1 = Simulink.data.dictionary.create('data/LA/LA_DataPkg_1.sldd');
end
importFromBaseWorkspace(LA_DataPkg_1, 'varList', {'busClass_Class_1'}, 'existingVarsAction','overwrite');
addDataSource(LA_DataPkg_1, 'LA_DataPkg_2.sldd');
saveChanges(LA_DataPkg_1);
clear LA_DataPkg_1;
file = proj.addFile('data/LA/LA_DataPkg_1.sldd');
addLabel(file, 'Capella Layer', 'LA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).Description = 'ModelerId:9f4a706d-d3a8-4235-ad85-c40a16048f26';
elems(1).DataType = 'Bus: busClass_Class_1';
elems(2) = Simulink.BusElement;
elems(2).Name = 'ExchangeItemElement_2';
elems(2).Description = 'ModelerId:b778486b-72eb-42c5-93d8-48b92815e2fb';
elems(2).DataType = 'single';
busEI_ExchangeItem_1b = Simulink.Bus;
busEI_ExchangeItem_1b.Description = 'ModelerId:3c73bd15-6607-4913-979c-0849ba566814';
busEI_ExchangeItem_1b.Elements = elems;
assignin('base', 'busEI_ExchangeItem_1b', busEI_ExchangeItem_1b);
clear elems;
NumericType_1 = Simulink.NumericType;
NumericType_1.DataTypeMode = 'Fixed-point: slope and bias scaling';
NumericType_1.Bias = 0;
NumericType_1.Signedness = 'Signed';
NumericType_1.WordLength = 64;
NumericType_1.IsAlias = true;
assignin('base', 'NumericType_1', NumericType_1);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).Description = 'ModelerId:48a0c374-b4ce-4fdf-92ba-4116932c2011';
elems(1).DataType = 'NumericType_1';
busEI_ExchangeItem_2 = Simulink.Bus;
busEI_ExchangeItem_2.Description = 'ModelerId:77a0c6b6-85c2-45cd-a1f2-f301012da059';
busEI_ExchangeItem_2.Elements = elems;
assignin('base', 'busEI_ExchangeItem_2', busEI_ExchangeItem_2);
clear elems;
if isfile('data/LA/LA_Data.sldd')
	LA_Data = Simulink.data.dictionary.open('data/LA/LA_Data.sldd');
else
	LA_Data = Simulink.data.dictionary.create('data/LA/LA_Data.sldd');
end
importFromBaseWorkspace(LA_Data, 'varList', {'busEI_ExchangeItem_1b', 'NumericType_1', 'busEI_ExchangeItem_2'}, 'existingVarsAction','overwrite');
addDataSource(LA_Data, 'LA_DataPkg_1.sldd');
saveChanges(LA_Data);
clear LA_Data;
file = proj.addFile('data/LA/LA_Data.sldd');
addLabel(file, 'Capella Layer', 'LA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_1b';
elems(1).Description = 'ModelerId:3c73bd15-6607-4913-979c-0849ba566814';
elems(1).DataType = 'Bus: busEI_ExchangeItem_1b';
elems(2) = Simulink.BusElement;
elems(2).Name = 'ExchangeItem_2';
elems(2).Description = 'ModelerId:77a0c6b6-85c2-45cd-a1f2-f301012da059';
elems(2).DataType = 'Bus: busEI_ExchangeItem_2';
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:84123cd2-c85a-454e-b080-1c451b470c89';
busFE_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
if isfile('data/LA/LA_FunctionalExchanges_Categories.sldd')
	LA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/LA/LA_FunctionalExchanges_Categories.sldd');
else
	LA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/LA/LA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(LA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_1'}, 'existingVarsAction','overwrite');
addDataSource(LA_FunctionalExchanges_Categories, 'LA_Data.sldd');
saveChanges(LA_FunctionalExchanges_Categories);
clear LA_FunctionalExchanges_Categories;
file = proj.addFile('data/LA/LA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_StateMachine_4.sldd')
	LA_StateMachine_4 = Simulink.data.dictionary.open('data/LA/LA_StateMachine_4.sldd');
else
	LA_StateMachine_4 = Simulink.data.dictionary.create('data/LA/LA_StateMachine_4.sldd');
end
addDataSource(LA_StateMachine_4, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_StateMachine_4);
clear LA_StateMachine_4;
file = proj.addFile('data/LA/LA_StateMachine_4.sldd');
addLabel(file, 'Capella Layer', 'LA');
evalin('base', "clear 'busClass_Class_1' 'Enumeration_2' 'busFE_FunctionalExchange_1' 'busEI_ExchangeItem_1b' 'NumericType_1' 'busEI_ExchangeItem_2'");
new_system('LA_StateMachine_4');
set_param('LA_StateMachine_4', 'Tag', 'ModelerId:c787ef3b-08a3-4c99-be0a-dda32f422246');
set_param('LA_StateMachine_4', 'DataDictionary', 'LA_StateMachine_4.sldd');
add_block('simulink/Ports & Subsystems/In1', 'LA_StateMachine_4/FunctionalExchange_1');
set_param('LA_StateMachine_4/FunctionalExchange_1', 'Port', '1');
set_param('LA_StateMachine_4/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('LA_StateMachine_4/FunctionalExchange_1', 'Tag', 'ModelerId:84123cd2-c85a-454e-b080-1c451b470c89');
set_param('LA_StateMachine_4/FunctionalExchange_1', 'BusOutputAsStruct', 'On')
add_block('simulink/Ports & Subsystems/In1', 'LA_StateMachine_4/ExchangeItem_1b');
set_param('LA_StateMachine_4/ExchangeItem_1b', 'Port', '2');
set_param('LA_StateMachine_4/ExchangeItem_1b', 'OutDataTypeStr', 'Bus: busEI_ExchangeItem_1b');
set_param('LA_StateMachine_4/ExchangeItem_1b', 'Tag', 'ModelerId:3c73bd15-6607-4913-979c-0849ba566814');
set_param('LA_StateMachine_4/ExchangeItem_1b', 'BusOutputAsStruct', 'On')
states = containers.Map;
add_block('sflib/Chart', 'LA_StateMachine_4/StateMachine_4');
set_param('LA_StateMachine_4/StateMachine_4', 'Tag', 'ModelerId:c787ef3b-08a3-4c99-be0a-dda32f422246');
rt = sfroot;
ch = rt.find('-isa','Stateflow.Chart','Path', 'LA_StateMachine_4/StateMachine_4');
myVar = Stateflow.Message(ch);
myVar.Name = 'FunctionalExchange_1_FE';
myVar.Scope = 'Input';
myVar.Port = 1;
myVar.Tag = 'ModelerId:84123cd2-c85a-454e-b080-1c451b470c89';
myVar.DataType = 'Bus: busFE_FunctionalExchange_1';
v = version('-release');
minor = v(5);
vers = str2num(v(1:4));
if (vers == 2019 && minor == 'b' ) | vers >= 2020;
	myVar.UseInternalQueue = 0;
end
clear v minor vers;
myVar = Stateflow.Message(ch);
myVar.Name = 'ExchangeItem_1b_EI';
myVar.Scope = 'Input';
myVar.Port = 2;
myVar.Tag = 'ModelerId:3c73bd15-6607-4913-979c-0849ba566814';
myVar.DataType = 'Bus: busEI_ExchangeItem_1b';
v = version('-release');
minor = v(5);
vers = str2num(v(1:4));
if (vers == 2019 && minor == 'b' ) | vers >= 2020;
	myVar.UseInternalQueue = 0;
end
clear v minor vers;
parent = ch;
state = Stateflow.State(parent);
state.Position = [410 30 52 67];
state.Tag = 'ModelerId:d758d18e-033f-42c0-ab92-b4e5c1307edf';
state.labelString = 'State_1';
state.Decomposition = 'EXCLUSIVE_OR';
states('d758d18e-033f-42c0-ab92-b4e5c1307edf') = state;
parent = ch;
state = Stateflow.State(parent);
state.Position = [30 40 52 67];
state.Tag = 'ModelerId:ac7e1d78-4920-4458-8fac-31fe3330980d';
state.labelString = 'State_2';
state.Decomposition = 'EXCLUSIVE_OR';
states('ac7e1d78-4920-4458-8fac-31fe3330980d') = state;
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:350bad8b-9b7b-41ab-a1b4-3c96667656f6';
source = states('ac7e1d78-4920-4458-8fac-31fe3330980d');
transition.source = source;
transition.SourceOClock = 4.459370825495623;
destination = states('d758d18e-033f-42c0-ab92-b4e5c1307edf');
transition.Destination = destination;
transition.DestinationOClock = 7.616784388159482;
transition.LabelString = ['FunctionalExchange_1_FE | ExchangeItem_1b_EI'];
clear states rt ch parent transition destination junction myVar source state;
handles = get_param('LA_StateMachine_4/FunctionalExchange_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LA_StateMachine_4/StateMachine_4', 'PortHandles');
target = handles.Inport(1);
line = add_line('LA_StateMachine_4', source, target, 'autorouting','smart');
clear line handles source target;
handles = get_param('LA_StateMachine_4/ExchangeItem_1b', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LA_StateMachine_4/StateMachine_4', 'PortHandles');
target = handles.Inport(2);
line = add_line('LA_StateMachine_4', source, target, 'autorouting','smart');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('LA_StateMachine_4');
catch
end
save_system('LA_StateMachine_4', fullfile(project_path, 'models/LA/LA_StateMachine_4.slx'));
file = proj.addFile('models/LA/LA_StateMachine_4.slx');
addLabel(file, 'Capella Layer', 'LA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
