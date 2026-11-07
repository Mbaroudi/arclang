if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject26');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject26', 'Folder', project_path);
end
cd(project_path);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Property_1';
elems(1).DataType = 'string';
elems(2) = Simulink.BusElement;
elems(2).Name = 'Property_2';
elems(2).DataType = 'string';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_ExchangeItem_1 = Simulink.Bus;
busEI_ExchangeItem_1.Elements = elems;
assignin('base', 'busEI_ExchangeItem_1', busEI_ExchangeItem_1);
clear elems;
if isfile('data/SA/SA_Data.sldd')
	SA_Data = Simulink.data.dictionary.open('data/SA/SA_Data.sldd');
else
	SA_Data = Simulink.data.dictionary.create('data/SA/SA_Data.sldd');
end
importFromBaseWorkspace(SA_Data, 'varList', {'busClass_Class_1', 'busEI_ExchangeItem_1'}, 'existingVarsAction','overwrite');
saveChanges(SA_Data);
clear SA_Data;
file = proj.addFile('data/SA/SA_Data.sldd');
addLabel(file, 'Capella Layer', 'SA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_1';
elems(1).DataType = 'Bus: busEI_ExchangeItem_1';
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
if isfile('data/SA/SA_FunctionalExchanges_Categories.sldd')
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/SA/SA_FunctionalExchanges_Categories.sldd');
else
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/SA/SA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(SA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_1'}, 'existingVarsAction','overwrite');
addDataSource(SA_FunctionalExchanges_Categories, 'SA_Data.sldd');
saveChanges(SA_FunctionalExchanges_Categories);
clear SA_FunctionalExchanges_Categories;
file = proj.addFile('data/SA/SA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_SystemFunction_1.sldd')
	SA_SystemFunction_1 = Simulink.data.dictionary.open('data/SA/SA_SystemFunction_1.sldd');
else
	SA_SystemFunction_1 = Simulink.data.dictionary.create('data/SA/SA_SystemFunction_1.sldd');
end
addDataSource(SA_SystemFunction_1, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_SystemFunction_1);
clear SA_SystemFunction_1;
file = proj.addFile('data/SA/SA_SystemFunction_1.sldd');
addLabel(file, 'Capella Layer', 'SA');
evalin('base', "clear 'busClass_Class_1' 'busFE_FunctionalExchange_1' 'busEI_ExchangeItem_1'");
new_system('SA_SystemFunction_1');
set_param('SA_SystemFunction_1', 'DataDictionary', 'SA_SystemFunction_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'SA_SystemFunction_1/FunctionalExchange_1');
set_param('SA_SystemFunction_1/FunctionalExchange_1', 'Port', '1');
set_param('SA_SystemFunction_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('SA_SystemFunction_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
g_Block = add_block('simulink/Sources/Ground', 'SA_SystemFunction_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('SA_SystemFunction_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('SA_SystemFunction_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('SA_SystemFunction_1');
catch
end
save_system('SA_SystemFunction_1', fullfile(project_path, 'models/SA/SA_SystemFunction_1.slx'));
file = proj.addFile('models/SA/SA_SystemFunction_1.slx');
addLabel(file, 'Capella Layer', 'SA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
