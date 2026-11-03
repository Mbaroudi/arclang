if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject27');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject27', 'Folder', project_path);
end
cd(project_path);
PhysicalQuantity_1 = Simulink.Signal;
PhysicalQuantity_1.DataType = 'int32';
PhysicalQuantity_1.Description = 'ModelerId:7787f6af-8440-4e4c-acbc-29e81a2bcdcd';
assignin('base', 'PhysicalQuantity_1', PhysicalQuantity_1);
if isfile('data/PA/PA_Data.sldd')
	PA_Data = Simulink.data.dictionary.open('data/PA/PA_Data.sldd');
else
	PA_Data = Simulink.data.dictionary.create('data/PA/PA_Data.sldd');
end
importFromBaseWorkspace(PA_Data, 'varList', {'PhysicalQuantity_1'}, 'existingVarsAction','overwrite');
saveChanges(PA_Data);
clear PA_Data;
file = proj.addFile('data/PA/PA_Data.sldd');
addLabel(file, 'Capella Layer', 'PA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).Description = 'ModelerId:53b272be-ad2b-402a-94a9-9d3d73f2c741';
elems(1).DataType = PhysicalQuantity_1.DataType;
busEI_ExchangeItem_1 = Simulink.Bus;
busEI_ExchangeItem_1.Description = 'ModelerId:126c60b1-357d-4710-a5c0-45925c09b584';
busEI_ExchangeItem_1.Elements = elems;
assignin('base', 'busEI_ExchangeItem_1', busEI_ExchangeItem_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_2';
elems(1).Description = 'ModelerId:69a69f5c-16ff-48f7-97fb-39af6381a638';
elems(1).DataType = PhysicalQuantity_1.DataType;
busEI_ExchangeItem_2 = Simulink.Bus;
busEI_ExchangeItem_2.Description = 'ModelerId:8391ab37-2d32-40af-a5cd-c4098186e45a';
busEI_ExchangeItem_2.Elements = elems;
assignin('base', 'busEI_ExchangeItem_2', busEI_ExchangeItem_2);
clear elems;
if isfile('data/LA/LA_Data.sldd')
	LA_Data = Simulink.data.dictionary.open('data/LA/LA_Data.sldd');
else
	LA_Data = Simulink.data.dictionary.create('data/LA/LA_Data.sldd');
end
importFromBaseWorkspace(LA_Data, 'varList', {'busEI_ExchangeItem_1', 'busEI_ExchangeItem_2'}, 'existingVarsAction','overwrite');
addDataSource(LA_Data, 'PA_Data.sldd');
saveChanges(LA_Data);
clear LA_Data;
file = proj.addFile('data/LA/LA_Data.sldd');
addLabel(file, 'Capella Layer', 'LA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_1';
elems(1).Description = 'ModelerId:126c60b1-357d-4710-a5c0-45925c09b584';
elems(1).DataType = 'Bus: busEI_ExchangeItem_1';
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:98a47e04-4476-4523-b467-2f9c5436b56e';
busFE_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_2';
elems(1).Description = 'ModelerId:8391ab37-2d32-40af-a5cd-c4098186e45a';
elems(1).DataType = 'Bus: busEI_ExchangeItem_2';
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:a068fe8b-4b95-48a3-932a-6ee322d9ab60';
busFE_FunctionalExchange_2.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
if isfile('data/LA/LA_FunctionalExchanges_Categories.sldd')
	LA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/LA/LA_FunctionalExchanges_Categories.sldd');
else
	LA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/LA/LA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(LA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_1', 'busFE_FunctionalExchange_2'}, 'existingVarsAction','overwrite');
addDataSource(LA_FunctionalExchanges_Categories, 'LA_Data.sldd');
saveChanges(LA_FunctionalExchanges_Categories);
clear LA_FunctionalExchanges_Categories;
file = proj.addFile('data/LA/LA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_LogicalFunction_1.sldd')
	LA_LogicalFunction_1 = Simulink.data.dictionary.open('data/LA/LA_LogicalFunction_1.sldd');
else
	LA_LogicalFunction_1 = Simulink.data.dictionary.create('data/LA/LA_LogicalFunction_1.sldd');
end
addDataSource(LA_LogicalFunction_1, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_LogicalFunction_1);
clear LA_LogicalFunction_1;
file = proj.addFile('data/LA/LA_LogicalFunction_1.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/SA/SA_Predefined_Types.sldd')
	SA_Predefined_Types = Simulink.data.dictionary.open('data/SA/SA_Predefined_Types.sldd');
else
	SA_Predefined_Types = Simulink.data.dictionary.create('data/SA/SA_Predefined_Types.sldd');
end
saveChanges(SA_Predefined_Types);
clear SA_Predefined_Types;
file = proj.addFile('data/SA/SA_Predefined_Types.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/LA/LA_LogicalFunction_2.sldd')
	LA_LogicalFunction_2 = Simulink.data.dictionary.open('data/LA/LA_LogicalFunction_2.sldd');
else
	LA_LogicalFunction_2 = Simulink.data.dictionary.create('data/LA/LA_LogicalFunction_2.sldd');
end
addDataSource(LA_LogicalFunction_2, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_LogicalFunction_2);
clear LA_LogicalFunction_2;
file = proj.addFile('data/LA/LA_LogicalFunction_2.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_LogicalFunction_3.sldd')
	LA_LogicalFunction_3 = Simulink.data.dictionary.open('data/LA/LA_LogicalFunction_3.sldd');
else
	LA_LogicalFunction_3 = Simulink.data.dictionary.create('data/LA/LA_LogicalFunction_3.sldd');
end
addDataSource(LA_LogicalFunction_3, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_LogicalFunction_3);
clear LA_LogicalFunction_3;
file = proj.addFile('data/LA/LA_LogicalFunction_3.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_FunctionalChain_1.sldd')
	LA_FunctionalChain_1 = Simulink.data.dictionary.open('data/LA/LA_FunctionalChain_1.sldd');
else
	LA_FunctionalChain_1 = Simulink.data.dictionary.create('data/LA/LA_FunctionalChain_1.sldd');
end
addDataSource(LA_FunctionalChain_1, 'LA_FunctionalExchanges_Categories.sldd');
saveChanges(LA_FunctionalChain_1);
clear LA_FunctionalChain_1;
file = proj.addFile('data/LA/LA_FunctionalChain_1.sldd');
addLabel(file, 'Capella Layer', 'LA');
evalin('base', "clear 'PhysicalQuantity_1' 'busFE_FunctionalExchange_1' 'busFE_FunctionalExchange_2' 'busEI_ExchangeItem_1' 'busEI_ExchangeItem_2'");
new_system('LA_LogicalFunction_3');
set_param('LA_LogicalFunction_3', 'Tag', 'ModelerId:7d8e21e9-20dc-4cdd-b49c-5be4d1ed5f6f');
set_param('LA_LogicalFunction_3', 'DataDictionary', 'LA_LogicalFunction_3.sldd');
add_block('simulink/Ports & Subsystems/In1', 'LA_LogicalFunction_3/FunctionalExchange_2');
set_param('LA_LogicalFunction_3/FunctionalExchange_2', 'Port', '1');
set_param('LA_LogicalFunction_3/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('LA_LogicalFunction_3/FunctionalExchange_2', 'Tag', 'ModelerId:414d3b80-97cc-4671-ba81-6e810c4389b8');
t_Block = add_block('simulink/Sinks/Terminator', 'LA_LogicalFunction_3/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_3/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_LogicalFunction_3', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('LA_LogicalFunction_3');
catch
end
save_system('LA_LogicalFunction_3', fullfile(project_path, 'models/LA/LA_LogicalFunction_3.slx'));
file = proj.addFile('models/LA/LA_LogicalFunction_3.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_LogicalFunction_2');
set_param('LA_LogicalFunction_2', 'Tag', 'ModelerId:8fa606a3-0a7c-4d91-a963-23b7d2c33f72');
set_param('LA_LogicalFunction_2', 'DataDictionary', 'LA_LogicalFunction_2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'LA_LogicalFunction_2/FunctionalExchange_2');
set_param('LA_LogicalFunction_2/FunctionalExchange_2', 'Port', '1');
set_param('LA_LogicalFunction_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('LA_LogicalFunction_2/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('LA_LogicalFunction_2/FunctionalExchange_2', 'Tag', 'ModelerId:2ff501f4-2070-4cff-931c-459aac18612f');
add_block('simulink/Ports & Subsystems/In1', 'LA_LogicalFunction_2/FunctionalExchange_1');
set_param('LA_LogicalFunction_2/FunctionalExchange_1', 'Port', '1');
set_param('LA_LogicalFunction_2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('LA_LogicalFunction_2/FunctionalExchange_1', 'Tag', 'ModelerId:25f1fd1e-54ef-4860-b864-e1e55fa81d07');
g_Block = add_block('simulink/Sources/Ground', 'LA_LogicalFunction_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('LA_LogicalFunction_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'LA_LogicalFunction_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_2/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_LogicalFunction_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('LA_LogicalFunction_2');
catch
end
save_system('LA_LogicalFunction_2', fullfile(project_path, 'models/LA/LA_LogicalFunction_2.slx'));
file = proj.addFile('models/LA/LA_LogicalFunction_2.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_LogicalFunction_1');
set_param('LA_LogicalFunction_1', 'Tag', 'ModelerId:49a3a98e-8275-4717-b923-702bea503c42');
set_param('LA_LogicalFunction_1', 'DataDictionary', 'LA_LogicalFunction_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'LA_LogicalFunction_1/FunctionalExchange_1');
set_param('LA_LogicalFunction_1/FunctionalExchange_1', 'Port', '1');
set_param('LA_LogicalFunction_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('LA_LogicalFunction_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('LA_LogicalFunction_1/FunctionalExchange_1', 'Tag', 'ModelerId:8cfbc7c5-cb4c-4924-811a-8141edb7ad13');
g_Block = add_block('simulink/Sources/Ground', 'LA_LogicalFunction_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('LA_LogicalFunction_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('LA_LogicalFunction_1');
catch
end
save_system('LA_LogicalFunction_1', fullfile(project_path, 'models/LA/LA_LogicalFunction_1.slx'));
file = proj.addFile('models/LA/LA_LogicalFunction_1.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_FunctionalChain_1');
set_param('LA_FunctionalChain_1', 'Tag', 'ModelerId:5b7a6332-77f6-47d2-94f7-040d4c0cdb21');
set_param('LA_FunctionalChain_1', 'DataDictionary', 'LA_FunctionalChain_1.sldd');
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_1/Reference_to_LogicalFunction_1');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_1', 'Tag', 'ModelerId:49a3a98e-8275-4717-b923-702bea503c42');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_1', 'ModelFile', 'LA_LogicalFunction_1')
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:8cfbc7c5-cb4c-4924-811a-8141edb7ad13');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_1/Reference_to_LogicalFunction_2');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'Tag', 'ModelerId:8fa606a3-0a7c-4d91-a963-23b7d2c33f72');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'ModelFile', 'LA_LogicalFunction_2')
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:2ff501f4-2070-4cff-931c-459aac18612f');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_1/Reference_to_LogicalFunction_3');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_3', 'Tag', 'ModelerId:7d8e21e9-20dc-4cdd-b49c-5be4d1ed5f6f');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_3', 'ModelFile', 'LA_LogicalFunction_3')
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_3', 'PortHandles');
target = handles.Inport(1);
line = add_line('LA_FunctionalChain_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:a068fe8b-4b95-48a3-932a-6ee322d9ab60');
clear line handles source target;
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('LA_FunctionalChain_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:98a47e04-4476-4523-b467-2f9c5436b56e');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('LA_FunctionalChain_1');
catch
end
save_system('LA_FunctionalChain_1', fullfile(project_path, 'models/LA/LA_FunctionalChain_1.slx'));
file = proj.addFile('models/LA/LA_FunctionalChain_1.slx');
addLabel(file, 'Capella Layer', 'LA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
