if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject19');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject19', 'Folder', project_path);
end
cd(project_path);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'id';
elems(1).Description = 'ModelerId:myOriginid';
elems(1).DataType = 'ID';
elems(2) = Simulink.BusElement;
elems(2).Name = 'Property_3';
elems(2).DataType = 'boolean';
busClass_Class_3 = Simulink.Bus;
busClass_Class_3.Description = 'ModelerId:myOriginClass3';
busClass_Class_3.Elements = elems;
assignin('base', 'busClass_Class_3', busClass_Class_3);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'class_3';
elems(1).DataType = 'Bus: busClass_Class_3';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Description = 'ModelerId:myOriginClass1';
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'class_1';
elems(1).DataType = 'Bus: busClass_Class_1';
elems(2) = Simulink.BusElement;
elems(2).Name = 'Property_2';
elems(2).DataType = 'int16';
busClass_Class_2 = Simulink.Bus;
busClass_Class_2.Description = 'ModelerId:myOriginClass2';
busClass_Class_2.Elements = elems;
assignin('base', 'busClass_Class_2', busClass_Class_2);
clear elems;
if isfile('data/SA/SA_DataPkg_3.sldd')
	SA_DataPkg_3 = Simulink.data.dictionary.open('data/SA/SA_DataPkg_3.sldd');
else
	SA_DataPkg_3 = Simulink.data.dictionary.create('data/SA/SA_DataPkg_3.sldd');
end
importFromBaseWorkspace(SA_DataPkg_3, 'varList', {'busClass_Class_3', 'busClass_Class_1', 'busClass_Class_2'}, 'existingVarsAction','overwrite');
saveChanges(SA_DataPkg_3);
clear SA_DataPkg_3;
file = proj.addFile('data/SA/SA_DataPkg_3.sldd');
addLabel(file, 'Capella Layer', 'SA');
Enumeration_1 = Simulink.data.dictionary.EnumTypeDefinition;
appendEnumeral(Enumeration_1, 'EnumerationLiteral_1', 0, '');
appendEnumeral(Enumeration_1, 'EnumerationLiteral_2', 1, '');
appendEnumeral(Enumeration_1, 'EnumerationLiteral_3', 2, '');
removeEnumeral(Enumeration_1,1);
assignin('base', 'Enumeration_1', Enumeration_1);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Property_1';
elems(1).Description = 'ModelerId:myOriginProperty1';
elems(1).DataType = 'Enum: Enumeration_1';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Description = 'ModelerId:myOriginClass1';
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
if isfile('data/SA/SA_DataPkg_4.sldd')
	SA_DataPkg_4 = Simulink.data.dictionary.open('data/SA/SA_DataPkg_4.sldd');
else
	SA_DataPkg_4 = Simulink.data.dictionary.create('data/SA/SA_DataPkg_4.sldd');
end
importFromBaseWorkspace(SA_DataPkg_4, 'varList', {'Enumeration_1', 'busClass_Class_1'}, 'existingVarsAction','overwrite');
saveChanges(SA_DataPkg_4);
clear SA_DataPkg_4;
file = proj.addFile('data/SA/SA_DataPkg_4.sldd');
addLabel(file, 'Capella Layer', 'SA');
ID = Simulink.NumericType;
assignin('base', 'ID', ID);
if isfile('data/SA/SA_DataPkg_2.sldd')
	SA_DataPkg_2 = Simulink.data.dictionary.open('data/SA/SA_DataPkg_2.sldd');
else
	SA_DataPkg_2 = Simulink.data.dictionary.create('data/SA/SA_DataPkg_2.sldd');
end
importFromBaseWorkspace(SA_DataPkg_2, 'varList', {'ID'}, 'existingVarsAction','overwrite');
saveChanges(SA_DataPkg_2);
clear SA_DataPkg_2;
file = proj.addFile('data/SA/SA_DataPkg_2.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/G_Head_Dictionary_.sldd')
	G_Head_Dictionary_ = Simulink.data.dictionary.open('data/G_Head_Dictionary_.sldd');
else
	G_Head_Dictionary_ = Simulink.data.dictionary.create('data/G_Head_Dictionary_.sldd');
end
addDataSource(G_Head_Dictionary_, 'SA_DataPkg_3.sldd');
addDataSource(G_Head_Dictionary_, 'SA_DataPkg_4.sldd');
addDataSource(G_Head_Dictionary_, 'SA_DataPkg_2.sldd');
saveChanges(G_Head_Dictionary_);
clear G_Head_Dictionary_;
file = proj.addFile('data/G_Head_Dictionary_.sldd');
evalin('base', "clear 'busClass_Class_1' 'busClass_Class_2' 'Enumeration_1' 'busClass_Class_3' 'ID'");
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
