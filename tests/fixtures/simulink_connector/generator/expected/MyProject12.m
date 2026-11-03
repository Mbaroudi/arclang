if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject', 'Folder', project_path);
end
cd(project_path);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
Project_dict_1_Bus = Simulink.Bus;
Project_dict_1_Bus.Elements = elems;
assignin('base', 'Project_dict_1_Bus', Project_dict_1_Bus);
clear elems;
if isfile('data/Project_dict_1.sldd')
	Project_dict_1 = Simulink.data.dictionary.open('data/Project_dict_1.sldd');
else
	Project_dict_1 = Simulink.data.dictionary.create('data/Project_dict_1.sldd');
end
importFromBaseWorkspace(Project_dict_1, 'varList', {'Project_dict_1_Bus'}, 'existingVarsAction','overwrite');
saveChanges(Project_dict_1);
clear Project_dict_1;
file = proj.addFile('data/Project_dict_1.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
Project_dict_2_Bus = Simulink.Bus;
Project_dict_2_Bus.Elements = elems;
assignin('base', 'Project_dict_2_Bus', Project_dict_2_Bus);
clear elems;
if isfile('data/Project_dict_2.sldd')
	Project_dict_2 = Simulink.data.dictionary.open('data/Project_dict_2.sldd');
else
	Project_dict_2 = Simulink.data.dictionary.create('data/Project_dict_2.sldd');
end
importFromBaseWorkspace(Project_dict_2, 'varList', {'Project_dict_2_Bus'}, 'existingVarsAction','overwrite');
saveChanges(Project_dict_2);
clear Project_dict_2;
file = proj.addFile('data/Project_dict_2.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
Project_dict_3_Bus = Simulink.Bus;
Project_dict_3_Bus.Elements = elems;
assignin('base', 'Project_dict_3_Bus', Project_dict_3_Bus);
clear elems;
if isfile('data/Project_dict_3.sldd')
	Project_dict_3 = Simulink.data.dictionary.open('data/Project_dict_3.sldd');
else
	Project_dict_3 = Simulink.data.dictionary.create('data/Project_dict_3.sldd');
end
importFromBaseWorkspace(Project_dict_3, 'varList', {'Project_dict_3_Bus'}, 'existingVarsAction','overwrite');
saveChanges(Project_dict_3);
clear Project_dict_3;
file = proj.addFile('data/Project_dict_3.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
sub_dict_1_1_Bus = Simulink.Bus;
sub_dict_1_1_Bus.Elements = elems;
assignin('base', 'sub_dict_1_1_Bus', sub_dict_1_1_Bus);
clear elems;
if isfile('data/sub_dict_1_1.sldd')
	sub_dict_1_1 = Simulink.data.dictionary.open('data/sub_dict_1_1.sldd');
else
	sub_dict_1_1 = Simulink.data.dictionary.create('data/sub_dict_1_1.sldd');
end
importFromBaseWorkspace(sub_dict_1_1, 'varList', {'sub_dict_1_1_Bus'}, 'existingVarsAction','overwrite');
saveChanges(sub_dict_1_1);
clear sub_dict_1_1;
file = proj.addFile('data/sub_dict_1_1.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
sub_dict_1_Bus = Simulink.Bus;
sub_dict_1_Bus.Elements = elems;
assignin('base', 'sub_dict_1_Bus', sub_dict_1_Bus);
clear elems;
if isfile('data/sub_dict_1.sldd')
	sub_dict_1 = Simulink.data.dictionary.open('data/sub_dict_1.sldd');
else
	sub_dict_1 = Simulink.data.dictionary.create('data/sub_dict_1.sldd');
end
importFromBaseWorkspace(sub_dict_1, 'varList', {'sub_dict_1_Bus'}, 'existingVarsAction','overwrite');
addDataSource(sub_dict_1, 'sub_dict_1_1.sldd');
saveChanges(sub_dict_1);
clear sub_dict_1;
file = proj.addFile('data/sub_dict_1.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
sub_dict_2_Bus = Simulink.Bus;
sub_dict_2_Bus.Elements = elems;
assignin('base', 'sub_dict_2_Bus', sub_dict_2_Bus);
clear elems;
myCustomType = Simulink.NumericType;
myCustomType.DataTypeMode = 'Fixed-point: slope and bias scaling';
myCustomType.WordLength = 32;
assignin('base', 'myCustomType', myCustomType);
if isfile('data/sub_dict_2.sldd')
	sub_dict_2 = Simulink.data.dictionary.open('data/sub_dict_2.sldd');
else
	sub_dict_2 = Simulink.data.dictionary.create('data/sub_dict_2.sldd');
end
importFromBaseWorkspace(sub_dict_2, 'varList', {'sub_dict_2_Bus', 'myCustomType'}, 'existingVarsAction','overwrite');
saveChanges(sub_dict_2);
clear sub_dict_2;
file = proj.addFile('data/sub_dict_2.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
sub_dict_3_1_Bus = Simulink.Bus;
sub_dict_3_1_Bus.Elements = elems;
assignin('base', 'sub_dict_3_1_Bus', sub_dict_3_1_Bus);
clear elems;
if isfile('data/sub_dict_3_1.sldd')
	sub_dict_3_1 = Simulink.data.dictionary.open('data/sub_dict_3_1.sldd');
else
	sub_dict_3_1 = Simulink.data.dictionary.create('data/sub_dict_3_1.sldd');
end
importFromBaseWorkspace(sub_dict_3_1, 'varList', {'sub_dict_3_1_Bus'}, 'existingVarsAction','overwrite');
saveChanges(sub_dict_3_1);
clear sub_dict_3_1;
file = proj.addFile('data/sub_dict_3_1.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
sub_dict_3_Bus = Simulink.Bus;
sub_dict_3_Bus.Elements = elems;
assignin('base', 'sub_dict_3_Bus', sub_dict_3_Bus);
clear elems;
if isfile('data/sub_dict_3.sldd')
	sub_dict_3 = Simulink.data.dictionary.open('data/sub_dict_3.sldd');
else
	sub_dict_3 = Simulink.data.dictionary.create('data/sub_dict_3.sldd');
end
importFromBaseWorkspace(sub_dict_3, 'varList', {'sub_dict_3_Bus'}, 'existingVarsAction','overwrite');
addDataSource(sub_dict_3, 'sub_dict_3_1.sldd');
saveChanges(sub_dict_3);
clear sub_dict_3;
file = proj.addFile('data/sub_dict_3.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
sub_dict_4_Bus = Simulink.Bus;
sub_dict_4_Bus.Elements = elems;
assignin('base', 'sub_dict_4_Bus', sub_dict_4_Bus);
clear elems;
if isfile('data/sub_dict_4.sldd')
	sub_dict_4 = Simulink.data.dictionary.open('data/sub_dict_4.sldd');
else
	sub_dict_4 = Simulink.data.dictionary.create('data/sub_dict_4.sldd');
end
importFromBaseWorkspace(sub_dict_4, 'varList', {'sub_dict_4_Bus'}, 'existingVarsAction','overwrite');
saveChanges(sub_dict_4);
clear sub_dict_4;
file = proj.addFile('data/sub_dict_4.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
System_dict_Bus = Simulink.Bus;
System_dict_Bus.Elements = elems;
assignin('base', 'System_dict_Bus', System_dict_Bus);
clear elems;
if isfile('data/System_dict.sldd')
	System_dict = Simulink.data.dictionary.open('data/System_dict.sldd');
else
	System_dict = Simulink.data.dictionary.create('data/System_dict.sldd');
end
importFromBaseWorkspace(System_dict, 'varList', {'System_dict_Bus'}, 'existingVarsAction','overwrite');
addDataSource(System_dict, 'sub_dict_1.sldd');
addDataSource(System_dict, 'sub_dict_2.sldd');
addDataSource(System_dict, 'sub_dict_3.sldd');
addDataSource(System_dict, 'sub_dict_4.sldd');
saveChanges(System_dict);
clear System_dict;
file = proj.addFile('data/System_dict.sldd');
evalin('base', "clear 'myCustomType' 'System_dict_Bus' 'sub_dict_4_Bus' 'Project_dict_1_Bus' 'sub_dict_3_Bus' 'sub_dict_2_Bus' 'Project_dict_2_Bus' 'Project_dict_3_Bus' 'sub_dict_1_1_Bus' 'sub_dict_1_Bus' 'sub_dict_3_1_Bus'");
new_system('MyModel');
set_param('MyModel', 'DataDictionary', 'System_dict.sldd');
add_block('simulink/Sources/Constant', 'MyModel/cst_1');
set_param('MyModel/cst_1', 'Position', '[10 10 35 35]')
set_param('MyModel/cst_1', 'OutDataTypeStr', 'myCustomType')
set_param('MyModel/cst_1', 'value', '1')
handles = get_param('MyModel/cst_1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyModel/cst_2');
set_param('MyModel/cst_2', 'Position', '[10 50 35 75]')
set_param('MyModel/cst_2', 'OutDataTypeStr', 'Bus: System_dict_Bus')
set_param('MyModel/cst_2', 'value', '0')
save_system('MyModel', fullfile(project_path, 'models/MyModel.slx'));
file = proj.addFile('models/MyModel.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
