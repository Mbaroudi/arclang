if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'pp');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'pp', 'Folder', project_path);
end
cd(project_path);
PhysicalQuantity_1 = Simulink.Signal;
PhysicalQuantity_1.Unit = 'm';
PhysicalQuantity_1.Description = 'ModelerId:ff9ea6c7-6ce1-41b4-9e65-c3cfe738740f';
assignin('base', 'PhysicalQuantity_1', PhysicalQuantity_1);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Property_1';
elems(1).Description = 'ModelerId:600749cb-c43c-4ba7-9ac1-c1e910ef08ae';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Description = 'ModelerId:51958f33-18c2-49ed-b782-210331341a85';
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'sub_ExchangeItemElement_1';
elems(1).Description = 'ModelerId:14443d90-d032-4a53-90b5-248cd67306a3';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_sub_ExchangeItem_1 = Simulink.Bus;
busEI_sub_ExchangeItem_1.Description = 'ModelerId:97d4d41f-e227-46f4-8cff-7316b2a316ef';
busEI_sub_ExchangeItem_1.Elements = elems;
assignin('base', 'busEI_sub_ExchangeItem_1', busEI_sub_ExchangeItem_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).Description = 'ModelerId:cd3f50bc-fb9f-48db-a389-5bffe74e78f5';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_ExchangeItem_1 = Simulink.Bus;
busEI_ExchangeItem_1.Description = 'ModelerId:040a0bdd-a53c-468b-94f1-6f7297808a82';
busEI_ExchangeItem_1.Elements = elems;
assignin('base', 'busEI_ExchangeItem_1', busEI_ExchangeItem_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).Description = 'ModelerId:3b9b0538-0732-44ab-8930-02bfd50f7510';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_ExchangeItem_2 = Simulink.Bus;
busEI_ExchangeItem_2.Description = 'ModelerId:ae5e8cac-d3ce-400d-8b19-f51a9cb74278';
busEI_ExchangeItem_2.Elements = elems;
assignin('base', 'busEI_ExchangeItem_2', busEI_ExchangeItem_2);
clear elems;
if isfile('data/PA/PA_Data.sldd')
	PA_Data = Simulink.data.dictionary.open('data/PA/PA_Data.sldd');
else
	PA_Data = Simulink.data.dictionary.create('data/PA/PA_Data.sldd');
end
importFromBaseWorkspace(PA_Data, 'varList', {'PhysicalQuantity_1', 'busClass_Class_1', 'busEI_sub_ExchangeItem_1', 'busEI_ExchangeItem_1', 'busEI_ExchangeItem_2'}, 'existingVarsAction','overwrite');
saveChanges(PA_Data);
clear PA_Data;
file = proj.addFile('data/PA/PA_Data.sldd');
addLabel(file, 'Capella Layer', 'PA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'sub_ExchangeItem_1';
elems(1).Description = 'ModelerId:97d4d41f-e227-46f4-8cff-7316b2a316ef';
elems(1).DataType = 'Bus: busEI_sub_ExchangeItem_1';
busFE_Sub_FunctionalExchange_1 = Simulink.Bus;
busFE_Sub_FunctionalExchange_1.Description = 'ModelerId:566e5d9f-f6bc-468a-9fd5-ded0f4a10c51';
busFE_Sub_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_Sub_FunctionalExchange_1', busFE_Sub_FunctionalExchange_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_1';
elems(1).Description = 'ModelerId:040a0bdd-a53c-468b-94f1-6f7297808a82';
elems(1).DataType = 'Bus: busEI_ExchangeItem_1';
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:9d5f8800-df02-4681-86c0-5680b7ebd444';
busFE_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_2';
elems(1).Description = 'ModelerId:ae5e8cac-d3ce-400d-8b19-f51a9cb74278';
elems(1).DataType = 'Bus: busEI_ExchangeItem_2';
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:8f69952d-36e4-46ae-bef9-d5dcd8138483';
busFE_FunctionalExchange_2.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
if isfile('data/PA/PA_FunctionalExchanges_Categories.sldd')
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/PA/PA_FunctionalExchanges_Categories.sldd');
else
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/PA/PA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(PA_FunctionalExchanges_Categories, 'varList', {'busFE_Sub_FunctionalExchange_1', 'busFE_FunctionalExchange_1', 'busFE_FunctionalExchange_2'}, 'existingVarsAction','overwrite');
addDataSource(PA_FunctionalExchanges_Categories, 'PA_Data.sldd');
saveChanges(PA_FunctionalExchanges_Categories);
clear PA_FunctionalExchanges_Categories;
file = proj.addFile('data/PA/PA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_sub_1.sldd')
	PA_sub_1 = Simulink.data.dictionary.open('data/PA/PA_sub_1.sldd');
else
	PA_sub_1 = Simulink.data.dictionary.create('data/PA/PA_sub_1.sldd');
end
addDataSource(PA_sub_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_sub_1);
clear PA_sub_1;
file = proj.addFile('data/PA/PA_sub_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_sub_2.sldd')
	PA_sub_2 = Simulink.data.dictionary.open('data/PA/PA_sub_2.sldd');
else
	PA_sub_2 = Simulink.data.dictionary.create('data/PA/PA_sub_2.sldd');
end
addDataSource(PA_sub_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_sub_2);
clear PA_sub_2;
file = proj.addFile('data/PA/PA_sub_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf_1.sldd')
	PA_leaf_1 = Simulink.data.dictionary.open('data/PA/PA_leaf_1.sldd');
else
	PA_leaf_1 = Simulink.data.dictionary.create('data/PA/PA_leaf_1.sldd');
end
addDataSource(PA_leaf_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf_1);
clear PA_leaf_1;
file = proj.addFile('data/PA/PA_leaf_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Leaf_2.sldd')
	PA_Leaf_2 = Simulink.data.dictionary.open('data/PA/PA_Leaf_2.sldd');
else
	PA_Leaf_2 = Simulink.data.dictionary.create('data/PA/PA_Leaf_2.sldd');
end
addDataSource(PA_Leaf_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_2);
clear PA_Leaf_2;
file = proj.addFile('data/PA/PA_Leaf_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_FunctionalChain_2.sldd')
	PA_FunctionalChain_2 = Simulink.data.dictionary.open('data/PA/PA_FunctionalChain_2.sldd');
else
	PA_FunctionalChain_2 = Simulink.data.dictionary.create('data/PA/PA_FunctionalChain_2.sldd');
end
addDataSource(PA_FunctionalChain_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_FunctionalChain_2);
clear PA_FunctionalChain_2;
file = proj.addFile('data/PA/PA_FunctionalChain_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
evalin('base', "clear 'PhysicalQuantity_1' 'busClass_Class_1' 'busEI_sub_ExchangeItem_1' 'busFE_FunctionalExchange_1' 'busFE_FunctionalExchange_2' 'busFE_Sub_FunctionalExchange_1' 'busEI_ExchangeItem_1' 'busEI_ExchangeItem_2'");
new_system('PA_Leaf_2');
set_param('PA_Leaf_2', 'Tag', 'ModelerId:4cadc4a0-eed1-4ed8-a5ef-e85838cc70ee');
set_param('PA_Leaf_2', 'DataDictionary', 'PA_Leaf_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Leaf_2/FunctionalExchange_2');
set_param('PA_Leaf_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_Leaf_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Leaf_2/FunctionalExchange_2', 'Tag', 'ModelerId:6540787d-e351-4218-b9a0-ee9ffec589ca');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Leaf_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Leaf_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_2');
catch
end
save_system('PA_Leaf_2', fullfile(project_path, 'models/PA/PA_Leaf_2.slx'));
file = proj.addFile('models/PA/PA_Leaf_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf_1');
set_param('PA_leaf_1', 'Tag', 'ModelerId:ffbf1867-6cbe-4c5b-ab0e-514ee01d5201');
set_param('PA_leaf_1', 'DataDictionary', 'PA_leaf_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_leaf_1/FunctionalExchange_1');
set_param('PA_leaf_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_leaf_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_leaf_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_leaf_1/FunctionalExchange_1', 'Tag', 'ModelerId:4a3f6acd-cb1d-4d89-b2d9-46d9d59f36e9');
g_Block = add_block('simulink/Sources/Ground', 'PA_leaf_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_leaf_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_leaf_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf_1');
catch
end
save_system('PA_leaf_1', fullfile(project_path, 'models/PA/PA_leaf_1.slx'));
file = proj.addFile('models/PA/PA_leaf_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_sub_2');
set_param('PA_sub_2', 'Tag', 'ModelerId:ee137cc4-d52d-484a-b685-16c449142824');
set_param('PA_sub_2', 'DataDictionary', 'PA_sub_2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_sub_2/FunctionalExchange_2');
set_param('PA_sub_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_sub_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_sub_2/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_sub_2/FunctionalExchange_2', 'Tag', 'ModelerId:171fe0da-d7f1-4b89-874b-3fcce7204f85');
add_block('simulink/Ports & Subsystems/In1', 'PA_sub_2/Sub_FunctionalExchange_1');
set_param('PA_sub_2/Sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_2/Sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_Sub_FunctionalExchange_1');
set_param('PA_sub_2/Sub_FunctionalExchange_1', 'Tag', 'ModelerId:79348269-33bd-4522-a8fb-21afeddd74bf');
g_Block = add_block('simulink/Sources/Ground', 'PA_sub_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_sub_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_sub_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_sub_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_sub_2/Sub_FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_sub_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_sub_2');
catch
end
save_system('PA_sub_2', fullfile(project_path, 'models/PA/PA_sub_2.slx'));
file = proj.addFile('models/PA/PA_sub_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_sub_1');
set_param('PA_sub_1', 'Tag', 'ModelerId:9d68ddf2-be8b-42f2-a28c-729f7fe949d1');
set_param('PA_sub_1', 'DataDictionary', 'PA_sub_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_sub_1/Sub_FunctionalExchange_1');
set_param('PA_sub_1/Sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_1/Sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_Sub_FunctionalExchange_1');
set_param('PA_sub_1/Sub_FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_sub_1/Sub_FunctionalExchange_1', 'Tag', 'ModelerId:8efaf64b-a45d-4cfc-9f40-1be624687e15');
add_block('simulink/Ports & Subsystems/In1', 'PA_sub_1/FunctionalExchange_1');
set_param('PA_sub_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_sub_1/FunctionalExchange_1', 'Tag', 'ModelerId:fb45b6cd-452e-48e5-94b8-a2c634d36d84');
g_Block = add_block('simulink/Sources/Ground', 'PA_sub_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_sub_1/Sub_FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_sub_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_sub_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_sub_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_sub_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_sub_1');
catch
end
save_system('PA_sub_1', fullfile(project_path, 'models/PA/PA_sub_1.slx'));
file = proj.addFile('models/PA/PA_sub_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_FunctionalChain_2');
set_param('PA_FunctionalChain_2', 'Tag', 'ModelerId:ce912255-452f-404a-b428-7b72841829a8');
set_param('PA_FunctionalChain_2', 'DataDictionary', 'PA_FunctionalChain_2.sldd');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_2/Reference_to_sub_1');
set_param('PA_FunctionalChain_2/Reference_to_sub_1', 'Tag', 'ModelerId:9d68ddf2-be8b-42f2-a28c-729f7fe949d1');
set_param('PA_FunctionalChain_2/Reference_to_sub_1', 'ModelFile', 'PA_sub_1')
handles = get_param('PA_FunctionalChain_2/Reference_to_sub_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:8efaf64b-a45d-4cfc-9f40-1be624687e15');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_2/Reference_to_sub_2');
set_param('PA_FunctionalChain_2/Reference_to_sub_2', 'Tag', 'ModelerId:ee137cc4-d52d-484a-b685-16c449142824');
set_param('PA_FunctionalChain_2/Reference_to_sub_2', 'ModelFile', 'PA_sub_2')
handles = get_param('PA_FunctionalChain_2/Reference_to_sub_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:171fe0da-d7f1-4b89-874b-3fcce7204f85');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_2/Reference_to_leaf_1');
set_param('PA_FunctionalChain_2/Reference_to_leaf_1', 'Tag', 'ModelerId:ffbf1867-6cbe-4c5b-ab0e-514ee01d5201');
set_param('PA_FunctionalChain_2/Reference_to_leaf_1', 'ModelFile', 'PA_leaf_1')
handles = get_param('PA_FunctionalChain_2/Reference_to_leaf_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:4a3f6acd-cb1d-4d89-b2d9-46d9d59f36e9');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_2/Reference_to_Leaf_2');
set_param('PA_FunctionalChain_2/Reference_to_Leaf_2', 'Tag', 'ModelerId:4cadc4a0-eed1-4ed8-a5ef-e85838cc70ee');
set_param('PA_FunctionalChain_2/Reference_to_Leaf_2', 'ModelFile', 'PA_Leaf_2')
handles = get_param('PA_FunctionalChain_2/Reference_to_sub_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_FunctionalChain_2/Reference_to_sub_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_FunctionalChain_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'Sub_FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:566e5d9f-f6bc-468a-9fd5-ded0f4a10c51');
clear line handles source target;
handles = get_param('PA_FunctionalChain_2/Reference_to_sub_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_FunctionalChain_2/Reference_to_Leaf_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_FunctionalChain_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:8f69952d-36e4-46ae-bef9-d5dcd8138483');
clear line handles source target;
handles = get_param('PA_FunctionalChain_2/Reference_to_leaf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_FunctionalChain_2/Reference_to_sub_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_FunctionalChain_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:9d5f8800-df02-4681-86c0-5680b7ebd444');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_FunctionalChain_2');
catch
end
save_system('PA_FunctionalChain_2', fullfile(project_path, 'models/PA/PA_FunctionalChain_2.slx'));
file = proj.addFile('models/PA/PA_FunctionalChain_2.slx');
addLabel(file, 'Capella Layer', 'PA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
