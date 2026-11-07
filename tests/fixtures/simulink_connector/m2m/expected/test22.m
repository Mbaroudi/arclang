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
evalin('base', "clear 'busFE_FunctionalExchange_1' 'busFE_FunctionalExchange_2'");
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
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
