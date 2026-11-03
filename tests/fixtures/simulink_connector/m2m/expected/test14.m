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
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:632b6ef3-c35a-41d2-997f-f80170fff7d4';
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
clear elems;
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:40af6851-833b-46fc-add0-95e6d433ab9f';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
busFE_FunctionalExchange_5 = Simulink.Bus;
busFE_FunctionalExchange_5.Description = 'ModelerId:4e8e5e1e-e889-4221-8c1a-3af14c740db5';
assignin('base', 'busFE_FunctionalExchange_5', busFE_FunctionalExchange_5);
clear elems;
clear elems;
busFE_FunctionalExchange_3 = Simulink.Bus;
busFE_FunctionalExchange_3.Description = 'ModelerId:46b3c1db-b44d-4667-bc1f-6a8d3b60fab2';
assignin('base', 'busFE_FunctionalExchange_3', busFE_FunctionalExchange_3);
clear elems;
clear elems;
busFE_FunctionalExchange_4 = Simulink.Bus;
busFE_FunctionalExchange_4.Description = 'ModelerId:4020f037-79ae-4ffb-be04-9ce7acc43a2e';
assignin('base', 'busFE_FunctionalExchange_4', busFE_FunctionalExchange_4);
clear elems;
if isfile('data/PA/PA_FunctionalExchanges_Categories.sldd')
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/PA/PA_FunctionalExchanges_Categories.sldd');
else
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/PA/PA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(PA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_2', 'busFE_FunctionalExchange_1', 'busFE_FunctionalExchange_5', 'busFE_FunctionalExchange_3', 'busFE_FunctionalExchange_4'}, 'existingVarsAction','overwrite');
saveChanges(PA_FunctionalExchanges_Categories);
clear PA_FunctionalExchanges_Categories;
file = proj.addFile('data/PA/PA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Container_1.sldd')
	PA_Container_1 = Simulink.data.dictionary.open('data/PA/PA_Container_1.sldd');
else
	PA_Container_1 = Simulink.data.dictionary.create('data/PA/PA_Container_1.sldd');
end
addDataSource(PA_Container_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Container_1);
clear PA_Container_1;
file = proj.addFile('data/PA/PA_Container_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Leaf_4.sldd')
	PA_Leaf_4 = Simulink.data.dictionary.open('data/PA/PA_Leaf_4.sldd');
else
	PA_Leaf_4 = Simulink.data.dictionary.create('data/PA/PA_Leaf_4.sldd');
end
addDataSource(PA_Leaf_4, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_4);
clear PA_Leaf_4;
file = proj.addFile('data/PA/PA_Leaf_4.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Leaf_1.sldd')
	PA_Leaf_1 = Simulink.data.dictionary.open('data/PA/PA_Leaf_1.sldd');
else
	PA_Leaf_1 = Simulink.data.dictionary.create('data/PA/PA_Leaf_1.sldd');
end
addDataSource(PA_Leaf_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_1);
clear PA_Leaf_1;
file = proj.addFile('data/PA/PA_Leaf_1.sldd');
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
if isfile('data/PA/PA_Leaf_3.sldd')
	PA_Leaf_3 = Simulink.data.dictionary.open('data/PA/PA_Leaf_3.sldd');
else
	PA_Leaf_3 = Simulink.data.dictionary.create('data/PA/PA_Leaf_3.sldd');
end
addDataSource(PA_Leaf_3, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_3);
clear PA_Leaf_3;
file = proj.addFile('data/PA/PA_Leaf_3.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Container_2.sldd')
	PA_Container_2 = Simulink.data.dictionary.open('data/PA/PA_Container_2.sldd');
else
	PA_Container_2 = Simulink.data.dictionary.create('data/PA/PA_Container_2.sldd');
end
addDataSource(PA_Container_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Container_2);
clear PA_Container_2;
file = proj.addFile('data/PA/PA_Container_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_FunctionalChain_3.sldd')
	PA_FunctionalChain_3 = Simulink.data.dictionary.open('data/PA/PA_FunctionalChain_3.sldd');
else
	PA_FunctionalChain_3 = Simulink.data.dictionary.create('data/PA/PA_FunctionalChain_3.sldd');
end
addDataSource(PA_FunctionalChain_3, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_FunctionalChain_3);
clear PA_FunctionalChain_3;
file = proj.addFile('data/PA/PA_FunctionalChain_3.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_FunctionalChain_4.sldd')
	PA_FunctionalChain_4 = Simulink.data.dictionary.open('data/PA/PA_FunctionalChain_4.sldd');
else
	PA_FunctionalChain_4 = Simulink.data.dictionary.create('data/PA/PA_FunctionalChain_4.sldd');
end
addDataSource(PA_FunctionalChain_4, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_FunctionalChain_4);
clear PA_FunctionalChain_4;
file = proj.addFile('data/PA/PA_FunctionalChain_4.sldd');
addLabel(file, 'Capella Layer', 'PA');
evalin('base', "clear 'busFE_FunctionalExchange_1' 'busFE_FunctionalExchange_3' 'busFE_FunctionalExchange_2' 'busFE_FunctionalExchange_5' 'busFE_FunctionalExchange_4'");
new_system('PA_Leaf_4');
set_param('PA_Leaf_4', 'Tag', 'ModelerId:b873a449-7c4e-4427-a059-a90dd4315e30');
set_param('PA_Leaf_4', 'DataDictionary', 'PA_Leaf_4.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Leaf_4/FunctionalExchange_4');
set_param('PA_Leaf_4/FunctionalExchange_4', 'Port', '1');
set_param('PA_Leaf_4/FunctionalExchange_4', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_4');
set_param('PA_Leaf_4/FunctionalExchange_4', 'Tag', 'ModelerId:45d9eef1-7e39-4df5-b99a-622df6fee4e6');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Leaf_4/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_4/FunctionalExchange_4', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Leaf_4', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_4');
catch
end
save_system('PA_Leaf_4', fullfile(project_path, 'models/PA/PA_Leaf_4.slx'));
file = proj.addFile('models/PA/PA_Leaf_4.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Leaf_3');
set_param('PA_Leaf_3', 'Tag', 'ModelerId:56c1f50d-ebad-4103-968f-25baf7557a2b');
set_param('PA_Leaf_3', 'DataDictionary', 'PA_Leaf_3.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Leaf_3/FunctionalExchange_5');
set_param('PA_Leaf_3/FunctionalExchange_5', 'Port', '1');
set_param('PA_Leaf_3/FunctionalExchange_5', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_5');
set_param('PA_Leaf_3/FunctionalExchange_5', 'BusOutputAsStruct', 'on');
set_param('PA_Leaf_3/FunctionalExchange_5', 'Tag', 'ModelerId:225d55f5-aa29-4389-97e4-feb621c8556f');
g_Block = add_block('simulink/Sources/Ground', 'PA_Leaf_3/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_3/FunctionalExchange_5', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Leaf_3', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_3');
catch
end
save_system('PA_Leaf_3', fullfile(project_path, 'models/PA/PA_Leaf_3.slx'));
file = proj.addFile('models/PA/PA_Leaf_3.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Container_2');
set_param('PA_Container_2', 'Tag', 'ModelerId:e7b08005-c709-4963-9826-d6188a34887f');
set_param('PA_Container_2', 'DataDictionary', 'PA_Container_2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Container_2/FunctionalExchange_3');
set_param('PA_Container_2/FunctionalExchange_3', 'Port', '1');
set_param('PA_Container_2/FunctionalExchange_3', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_3');
set_param('PA_Container_2/FunctionalExchange_3', 'BusOutputAsStruct', 'on');
set_param('PA_Container_2/FunctionalExchange_3', 'Tag', 'ModelerId:b71f74d4-36d7-4987-853a-dd60711e53f0');
add_block('simulink/Ports & Subsystems/In1', 'PA_Container_2/FunctionalExchange_2');
set_param('PA_Container_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_Container_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Container_2/FunctionalExchange_2', 'Tag', 'ModelerId:74132534-ba77-45c2-a616-ddd95d67480d');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Container_2/FunctionalExchange_4');
set_param('PA_Container_2/FunctionalExchange_4', 'Port', '2');
set_param('PA_Container_2/FunctionalExchange_4', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_4');
set_param('PA_Container_2/FunctionalExchange_4', 'BusOutputAsStruct', 'on');
set_param('PA_Container_2/FunctionalExchange_4', 'Tag', 'ModelerId:92b2407a-330e-4007-97d7-60dd613d4f2e');
g_Block = add_block('simulink/Sources/Ground', 'PA_Container_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Container_2/FunctionalExchange_3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Container_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
g_Block = add_block('simulink/Sources/Ground', 'PA_Container_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Container_2/FunctionalExchange_4', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Container_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Container_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Container_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Container_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Container_2');
catch
end
save_system('PA_Container_2', fullfile(project_path, 'models/PA/PA_Container_2.slx'));
file = proj.addFile('models/PA/PA_Container_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Container_1');
set_param('PA_Container_1', 'Tag', 'ModelerId:385587e8-9c47-4d4c-9576-f35641c14163');
set_param('PA_Container_1', 'DataDictionary', 'PA_Container_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Container_1/FunctionalExchange_2');
set_param('PA_Container_1/FunctionalExchange_2', 'Port', '1');
set_param('PA_Container_1/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Container_1/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_Container_1/FunctionalExchange_2', 'Tag', 'ModelerId:fc925960-f543-4069-9182-7b2f4c2e0351');
add_block('simulink/Ports & Subsystems/In1', 'PA_Container_1/FunctionalExchange_1');
set_param('PA_Container_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_Container_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_Container_1/FunctionalExchange_1', 'Tag', 'ModelerId:6022bc1d-a918-4046-a5c3-add21b5551ba');
add_block('simulink/Ports & Subsystems/In1', 'PA_Container_1/FunctionalExchange_5');
set_param('PA_Container_1/FunctionalExchange_5', 'Port', '2');
set_param('PA_Container_1/FunctionalExchange_5', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_5');
set_param('PA_Container_1/FunctionalExchange_5', 'Tag', 'ModelerId:e910048a-c4fa-4e91-97e0-753b847d3018');
g_Block = add_block('simulink/Sources/Ground', 'PA_Container_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Container_1/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Container_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Container_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Container_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Container_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Container_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Container_1/FunctionalExchange_5', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Container_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Container_1');
catch
end
save_system('PA_Container_1', fullfile(project_path, 'models/PA/PA_Container_1.slx'));
file = proj.addFile('models/PA/PA_Container_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_FunctionalChain_4');
set_param('PA_FunctionalChain_4', 'Tag', 'ModelerId:96c378fc-e4af-42fa-9d96-e1b6453162e6');
set_param('PA_FunctionalChain_4', 'DataDictionary', 'PA_FunctionalChain_4.sldd');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_4/Reference_to_Container_1');
set_param('PA_FunctionalChain_4/Reference_to_Container_1', 'Tag', 'ModelerId:385587e8-9c47-4d4c-9576-f35641c14163');
set_param('PA_FunctionalChain_4/Reference_to_Container_1', 'ModelFile', 'PA_Container_1')
handles = get_param('PA_FunctionalChain_4/Reference_to_Container_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:fc925960-f543-4069-9182-7b2f4c2e0351');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_4/Reference_to_Container_2');
set_param('PA_FunctionalChain_4/Reference_to_Container_2', 'Tag', 'ModelerId:e7b08005-c709-4963-9826-d6188a34887f');
set_param('PA_FunctionalChain_4/Reference_to_Container_2', 'ModelFile', 'PA_Container_2')
handles = get_param('PA_FunctionalChain_4/Reference_to_Container_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:b71f74d4-36d7-4987-853a-dd60711e53f0');
clear handles;
handles = get_param('PA_FunctionalChain_4/Reference_to_Container_2', 'porthandles');
set_param(handles.Outport(2), 'Tag', 'ModelerId:92b2407a-330e-4007-97d7-60dd613d4f2e');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_4/Reference_to_Leaf_3');
set_param('PA_FunctionalChain_4/Reference_to_Leaf_3', 'Tag', 'ModelerId:56c1f50d-ebad-4103-968f-25baf7557a2b');
set_param('PA_FunctionalChain_4/Reference_to_Leaf_3', 'ModelFile', 'PA_Leaf_3')
handles = get_param('PA_FunctionalChain_4/Reference_to_Leaf_3', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:225d55f5-aa29-4389-97e4-feb621c8556f');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_4/Reference_to_Leaf_4');
set_param('PA_FunctionalChain_4/Reference_to_Leaf_4', 'Tag', 'ModelerId:b873a449-7c4e-4427-a059-a90dd4315e30');
set_param('PA_FunctionalChain_4/Reference_to_Leaf_4', 'ModelFile', 'PA_Leaf_4')
handles = get_param('PA_FunctionalChain_4/Reference_to_Container_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_FunctionalChain_4/Reference_to_Container_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_FunctionalChain_4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:632b6ef3-c35a-41d2-997f-f80170fff7d4');
clear line handles source target;
handles = get_param('PA_FunctionalChain_4/Reference_to_Leaf_3', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_FunctionalChain_4/Reference_to_Container_1', 'PortHandles');
target = handles.Inport(2);
line = add_line('PA_FunctionalChain_4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_5');
set_param( line, 'Tag', 'ModelerId:4e8e5e1e-e889-4221-8c1a-3af14c740db5');
clear line handles source target;
handles = get_param('PA_FunctionalChain_4/Reference_to_Container_2', 'PortHandles');
source = handles.Outport(2);
handles = get_param('PA_FunctionalChain_4/Reference_to_Leaf_4', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_FunctionalChain_4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_4');
set_param( line, 'Tag', 'ModelerId:4020f037-79ae-4ffb-be04-9ce7acc43a2e');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_FunctionalChain_4');
catch
end
save_system('PA_FunctionalChain_4', fullfile(project_path, 'models/PA/PA_FunctionalChain_4.slx'));
file = proj.addFile('models/PA/PA_FunctionalChain_4.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Leaf_2');
set_param('PA_Leaf_2', 'Tag', 'ModelerId:b25c4b83-b531-41ff-87f8-a8b686dba851');
set_param('PA_Leaf_2', 'DataDictionary', 'PA_Leaf_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Leaf_2/FunctionalExchange_3');
set_param('PA_Leaf_2/FunctionalExchange_3', 'Port', '1');
set_param('PA_Leaf_2/FunctionalExchange_3', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_3');
set_param('PA_Leaf_2/FunctionalExchange_3', 'Tag', 'ModelerId:cb03c618-d503-41f1-bf4f-758c67514f24');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Leaf_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_2/FunctionalExchange_3', 'handle');
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
new_system('PA_Leaf_1');
set_param('PA_Leaf_1', 'Tag', 'ModelerId:5afe0c00-0313-479a-976f-b7771b500c94');
set_param('PA_Leaf_1', 'DataDictionary', 'PA_Leaf_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Leaf_1/FunctionalExchange_1');
set_param('PA_Leaf_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_Leaf_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_Leaf_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_Leaf_1/FunctionalExchange_1', 'Tag', 'ModelerId:24869e7d-82a3-4486-89e0-d4b7ab52ca54');
g_Block = add_block('simulink/Sources/Ground', 'PA_Leaf_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Leaf_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_1');
catch
end
save_system('PA_Leaf_1', fullfile(project_path, 'models/PA/PA_Leaf_1.slx'));
file = proj.addFile('models/PA/PA_Leaf_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_FunctionalChain_3');
set_param('PA_FunctionalChain_3', 'Tag', 'ModelerId:5e0a5f3d-e7ee-456d-80e5-4b79d6794a61');
set_param('PA_FunctionalChain_3', 'DataDictionary', 'PA_FunctionalChain_3.sldd');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_3/Reference_to_Container_1');
set_param('PA_FunctionalChain_3/Reference_to_Container_1', 'Tag', 'ModelerId:385587e8-9c47-4d4c-9576-f35641c14163');
set_param('PA_FunctionalChain_3/Reference_to_Container_1', 'ModelFile', 'PA_Container_1')
handles = get_param('PA_FunctionalChain_3/Reference_to_Container_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:fc925960-f543-4069-9182-7b2f4c2e0351');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_3/Reference_to_Container_2');
set_param('PA_FunctionalChain_3/Reference_to_Container_2', 'Tag', 'ModelerId:e7b08005-c709-4963-9826-d6188a34887f');
set_param('PA_FunctionalChain_3/Reference_to_Container_2', 'ModelFile', 'PA_Container_2')
handles = get_param('PA_FunctionalChain_3/Reference_to_Container_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:b71f74d4-36d7-4987-853a-dd60711e53f0');
clear handles;
handles = get_param('PA_FunctionalChain_3/Reference_to_Container_2', 'porthandles');
set_param(handles.Outport(2), 'Tag', 'ModelerId:92b2407a-330e-4007-97d7-60dd613d4f2e');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_3/Reference_to_Leaf_1');
set_param('PA_FunctionalChain_3/Reference_to_Leaf_1', 'Tag', 'ModelerId:5afe0c00-0313-479a-976f-b7771b500c94');
set_param('PA_FunctionalChain_3/Reference_to_Leaf_1', 'ModelFile', 'PA_Leaf_1')
handles = get_param('PA_FunctionalChain_3/Reference_to_Leaf_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:24869e7d-82a3-4486-89e0-d4b7ab52ca54');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_FunctionalChain_3/Reference_to_Leaf_2');
set_param('PA_FunctionalChain_3/Reference_to_Leaf_2', 'Tag', 'ModelerId:b25c4b83-b531-41ff-87f8-a8b686dba851');
set_param('PA_FunctionalChain_3/Reference_to_Leaf_2', 'ModelFile', 'PA_Leaf_2')
handles = get_param('PA_FunctionalChain_3/Reference_to_Leaf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_FunctionalChain_3/Reference_to_Container_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_FunctionalChain_3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:40af6851-833b-46fc-add0-95e6d433ab9f');
clear line handles source target;
handles = get_param('PA_FunctionalChain_3/Reference_to_Container_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_FunctionalChain_3/Reference_to_Container_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_FunctionalChain_3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:632b6ef3-c35a-41d2-997f-f80170fff7d4');
clear line handles source target;
handles = get_param('PA_FunctionalChain_3/Reference_to_Container_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_FunctionalChain_3/Reference_to_Leaf_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_FunctionalChain_3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_3');
set_param( line, 'Tag', 'ModelerId:46b3c1db-b44d-4667-bc1f-6a8d3b60fab2');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_FunctionalChain_3');
catch
end
save_system('PA_FunctionalChain_3', fullfile(project_path, 'models/PA/PA_FunctionalChain_3.slx'));
file = proj.addFile('models/PA/PA_FunctionalChain_3.slx');
addLabel(file, 'Capella Layer', 'PA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
