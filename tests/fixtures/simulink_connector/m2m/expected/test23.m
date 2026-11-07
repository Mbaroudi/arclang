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
clear elems;
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:c516ee37-5af0-4174-949e-e21f59d7ead8';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:e4a796d2-c321-43fb-a83b-6db60018bde4';
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
if isfile('data/SA/SA_FunctionalExchanges_Categories.sldd')
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/SA/SA_FunctionalExchanges_Categories.sldd');
else
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/SA/SA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(SA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_1', 'busFE_FunctionalExchange_2'}, 'existingVarsAction','overwrite');
saveChanges(SA_FunctionalExchanges_Categories);
clear SA_FunctionalExchanges_Categories;
file = proj.addFile('data/SA/SA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_End.sldd')
	SA_End = Simulink.data.dictionary.open('data/SA/SA_End.sldd');
else
	SA_End = Simulink.data.dictionary.create('data/SA/SA_End.sldd');
end
addDataSource(SA_End, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_End);
clear SA_End;
file = proj.addFile('data/SA/SA_End.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_F1.sldd')
	SA_F1 = Simulink.data.dictionary.open('data/SA/SA_F1.sldd');
else
	SA_F1 = Simulink.data.dictionary.create('data/SA/SA_F1.sldd');
end
addDataSource(SA_F1, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_F1);
clear SA_F1;
file = proj.addFile('data/SA/SA_F1.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_F2.sldd')
	SA_F2 = Simulink.data.dictionary.open('data/SA/SA_F2.sldd');
else
	SA_F2 = Simulink.data.dictionary.create('data/SA/SA_F2.sldd');
end
addDataSource(SA_F2, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_F2);
clear SA_F2;
file = proj.addFile('data/SA/SA_F2.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_Top2.sldd')
	SA_Top2 = Simulink.data.dictionary.open('data/SA/SA_Top2.sldd');
else
	SA_Top2 = Simulink.data.dictionary.create('data/SA/SA_Top2.sldd');
end
addDataSource(SA_Top2, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_Top2);
clear SA_Top2;
file = proj.addFile('data/SA/SA_Top2.sldd');
addLabel(file, 'Capella Layer', 'SA');
evalin('base', "clear 'busFE_FunctionalExchange_1' 'busFE_FunctionalExchange_2'");
new_system('SA_F2');
set_param('SA_F2', 'Tag', 'ModelerId:429a58e4-4e1e-440d-b392-fdb724707385');
set_param('SA_F2', 'DataDictionary', 'SA_F2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'SA_F2/FunctionalExchange_2');
set_param('SA_F2/FunctionalExchange_2', 'Port', '1');
set_param('SA_F2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('SA_F2/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('SA_F2/FunctionalExchange_2', 'Tag', 'ModelerId:c028547f-51d2-4cec-a315-a4183a2cac65');
g_Block = add_block('simulink/Sources/Ground', 'SA_F2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('SA_F2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('SA_F2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('SA_F2');
catch
end
save_system('SA_F2', fullfile(project_path, 'models/SA/SA_F2.slx'));
file = proj.addFile('models/SA/SA_F2.slx');
addLabel(file, 'Capella Layer', 'SA');
new_system('SA_F1');
set_param('SA_F1', 'Tag', 'ModelerId:e644082f-ee2f-454e-afbe-242d14107cc4');
set_param('SA_F1', 'DataDictionary', 'SA_F1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'SA_F1/FunctionalExchange_1');
set_param('SA_F1/FunctionalExchange_1', 'Port', '1');
set_param('SA_F1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('SA_F1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('SA_F1/FunctionalExchange_1', 'Tag', 'ModelerId:17f6b379-dc0e-46cf-8d51-d108548bee2c');
g_Block = add_block('simulink/Sources/Ground', 'SA_F1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('SA_F1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('SA_F1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('SA_F1');
catch
end
save_system('SA_F1', fullfile(project_path, 'models/SA/SA_F1.slx'));
file = proj.addFile('models/SA/SA_F1.slx');
addLabel(file, 'Capella Layer', 'SA');
new_system('SA_End');
set_param('SA_End', 'Tag', 'ModelerId:1495d8eb-966b-4427-a643-a6b2f1c8506f');
set_param('SA_End', 'DataDictionary', 'SA_End.sldd');
add_block('simulink/Ports & Subsystems/In1', 'SA_End/FunctionalExchange_1');
set_param('SA_End/FunctionalExchange_1', 'Port', '1');
set_param('SA_End/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('SA_End/FunctionalExchange_1', 'Tag', 'ModelerId:daee8b5d-eb74-44b7-a7b2-9590418ad3e2');
add_block('simulink/Ports & Subsystems/In1', 'SA_End/FunctionalExchange_1_1');
set_param('SA_End/FunctionalExchange_1_1', 'Port', '2');
set_param('SA_End/FunctionalExchange_1_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('SA_End/FunctionalExchange_1_1', 'Tag', 'ModelerId:daee8b5d-eb74-44b7-a7b2-9590418ad3e2');
t_Block = add_block('simulink/Sinks/Terminator', 'SA_End/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('SA_End/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('SA_End', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'SA_End/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('SA_End/FunctionalExchange_1_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('SA_End', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('SA_End');
catch
end
save_system('SA_End', fullfile(project_path, 'models/SA/SA_End.slx'));
file = proj.addFile('models/SA/SA_End.slx');
addLabel(file, 'Capella Layer', 'SA');
new_system('SA_Top2');
set_param('SA_Top2', 'Tag', 'ModelerId:fd405e78-af32-4532-807d-ff56710ad679');
set_param('SA_Top2', 'DataDictionary', 'SA_Top2.sldd');
add_block('CapellaElements/ModelRef/Model_Function', 'SA_Top2/Reference_to_End');
set_param('SA_Top2/Reference_to_End', 'Tag', 'ModelerId:1495d8eb-966b-4427-a643-a6b2f1c8506f');
set_param('SA_Top2/Reference_to_End', 'ModelFile', 'SA_End')
add_block('CapellaElements/ModelRef/Model_Function', 'SA_Top2/Reference_to_F1');
set_param('SA_Top2/Reference_to_F1', 'Tag', 'ModelerId:e644082f-ee2f-454e-afbe-242d14107cc4');
set_param('SA_Top2/Reference_to_F1', 'ModelFile', 'SA_F1')
handles = get_param('SA_Top2/Reference_to_F1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:17f6b379-dc0e-46cf-8d51-d108548bee2c');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'SA_Top2/Reference_to_F2');
set_param('SA_Top2/Reference_to_F2', 'Tag', 'ModelerId:429a58e4-4e1e-440d-b392-fdb724707385');
set_param('SA_Top2/Reference_to_F2', 'ModelFile', 'SA_F2')
handles = get_param('SA_Top2/Reference_to_F2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:c028547f-51d2-4cec-a315-a4183a2cac65');
clear handles;
handles = get_param('SA_Top2/Reference_to_F1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('SA_Top2/Reference_to_End', 'PortHandles');
target = handles.Inport(1);
line = add_line('SA_Top2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:c516ee37-5af0-4174-949e-e21f59d7ead8');
clear line handles source target;
handles = get_param('SA_Top2/Reference_to_F2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('SA_Top2/Reference_to_End', 'PortHandles');
target = handles.Inport(2);
line = add_line('SA_Top2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:e4a796d2-c321-43fb-a83b-6db60018bde4');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('SA_Top2');
catch
end
save_system('SA_Top2', fullfile(project_path, 'models/SA/SA_Top2.slx'));
file = proj.addFile('models/SA/SA_Top2.slx');
addLabel(file, 'Capella Layer', 'SA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
