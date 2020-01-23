var searchIndex={};
searchIndex["mimicaw"] = {"doc":"A library for writing asynchronous tests.","i":[[3,"Args","mimicaw","Command line arguments.",null,null],[12,"list","","",0,null],[12,"filter","","",0,null],[12,"filter_exact","","",0,null],[12,"run_ignored","","",0,null],[12,"run_tests","","",0,null],[12,"run_benchmarks","","",0,null],[12,"logfile","","",0,null],[12,"nocapture","","",0,null],[12,"color","","",0,null],[12,"format","","",0,null],[12,"test_threads","","",0,null],[12,"skip","","",0,null],[3,"Outcome","","The outcome of performing a test.",null,null],[3,"Test","","Data that describes a single test.",null,null],[3,"TestDesc","","Description about a test.",null,null],[3,"ExitStatus","","Exit status code used as a result of the test process.",null,null],[4,"ColorConfig","","The color configuration.",null,null],[13,"Auto","","",1,null],[13,"Always","","",1,null],[13,"Never","","",1,null],[4,"OutputFormat","","The output format.",null,null],[13,"Pretty","","",2,null],[13,"Terse","","",2,null],[13,"Json","","",2,null],[5,"run_tests","","Run a set of tests.",null,[[["args"]]]],[11,"from_env","","Parse command line arguments.",0,[[],[["result",["exitstatus"]],["exitstatus"]]]],[11,"name","","Return the name of test.",3,[[["self"]],["str"]]],[11,"is_bench","","Return whether the test is a benchmark or not.",3,[[["self"]],["bool"]]],[11,"ignored","","Return whether the test should be ignored or not.",3,[[["self"]],["bool"]]],[11,"test","","Create a single test.",4,[[["str"],["d"]],["self"]]],[11,"bench","","Create a single benchmark test.",4,[[["str"],["d"]],["self"]]],[11,"ignore","","Mark that this test should be ignored.",4,[[["bool"]],["self"]]],[11,"passed","","Create an `Outcome` representing that the test passed.",5,[[],["self"]]],[11,"failed","","Create an `Outcome` representing that the test or…",5,[[],["self"]]],[11,"measured","","Create an `Outcome` representing that the benchmark test…",5,[[["u64"]],["self"]]],[11,"error_message","","Specify the error message.",5,[[],["self"]]],[11,"success","","Return whether the status is successful or not.",6,[[],["bool"]]],[11,"code","","Return the raw exit code.",6,[[],["i32"]]],[11,"exit","","Terminate the test process with the exit code.",6,[[]]],[11,"exit_if_failed","","Terminate the test process if the exit code is not…",6,[[]]],[11,"from","","",0,[[["t"]],["t"]]],[11,"into","","",0,[[],["u"]]],[11,"try_from","","",0,[[["u"]],["result"]]],[11,"try_into","","",0,[[],["result"]]],[11,"borrow","","",0,[[["self"]],["t"]]],[11,"borrow_mut","","",0,[[["self"]],["t"]]],[11,"type_id","","",0,[[["self"]],["typeid"]]],[11,"from","","",5,[[["t"]],["t"]]],[11,"into","","",5,[[],["u"]]],[11,"try_from","","",5,[[["u"]],["result"]]],[11,"try_into","","",5,[[],["result"]]],[11,"borrow","","",5,[[["self"]],["t"]]],[11,"borrow_mut","","",5,[[["self"]],["t"]]],[11,"type_id","","",5,[[["self"]],["typeid"]]],[11,"from","","",4,[[["t"]],["t"]]],[11,"into","","",4,[[],["u"]]],[11,"try_from","","",4,[[["u"]],["result"]]],[11,"try_into","","",4,[[],["result"]]],[11,"borrow","","",4,[[["self"]],["t"]]],[11,"borrow_mut","","",4,[[["self"]],["t"]]],[11,"type_id","","",4,[[["self"]],["typeid"]]],[11,"from","","",3,[[["t"]],["t"]]],[11,"into","","",3,[[],["u"]]],[11,"to_owned","","",3,[[["self"]],["t"]]],[11,"clone_into","","",3,[[["self"],["t"]]]],[11,"try_from","","",3,[[["u"]],["result"]]],[11,"try_into","","",3,[[],["result"]]],[11,"borrow","","",3,[[["self"]],["t"]]],[11,"borrow_mut","","",3,[[["self"]],["t"]]],[11,"type_id","","",3,[[["self"]],["typeid"]]],[11,"from","","",6,[[["t"]],["t"]]],[11,"into","","",6,[[],["u"]]],[11,"to_owned","","",6,[[["self"]],["t"]]],[11,"clone_into","","",6,[[["self"],["t"]]]],[11,"try_from","","",6,[[["u"]],["result"]]],[11,"try_into","","",6,[[],["result"]]],[11,"borrow","","",6,[[["self"]],["t"]]],[11,"borrow_mut","","",6,[[["self"]],["t"]]],[11,"type_id","","",6,[[["self"]],["typeid"]]],[11,"from","","",1,[[["t"]],["t"]]],[11,"into","","",1,[[],["u"]]],[11,"to_owned","","",1,[[["self"]],["t"]]],[11,"clone_into","","",1,[[["self"],["t"]]]],[11,"try_from","","",1,[[["u"]],["result"]]],[11,"try_into","","",1,[[],["result"]]],[11,"borrow","","",1,[[["self"]],["t"]]],[11,"borrow_mut","","",1,[[["self"]],["t"]]],[11,"type_id","","",1,[[["self"]],["typeid"]]],[11,"from","","",2,[[["t"]],["t"]]],[11,"into","","",2,[[],["u"]]],[11,"to_owned","","",2,[[["self"]],["t"]]],[11,"clone_into","","",2,[[["self"],["t"]]]],[11,"try_from","","",2,[[["u"]],["result"]]],[11,"try_into","","",2,[[],["result"]]],[11,"borrow","","",2,[[["self"]],["t"]]],[11,"borrow_mut","","",2,[[["self"]],["t"]]],[11,"type_id","","",2,[[["self"]],["typeid"]]],[11,"as_ref","","",3,[[["self"]],["self"]]],[11,"clone","","",1,[[["self"]],["colorconfig"]]],[11,"clone","","",2,[[["self"]],["outputformat"]]],[11,"clone","","",3,[[["self"]],["testdesc"]]],[11,"clone","","",6,[[["self"]],["exitstatus"]]],[11,"eq","","",1,[[["self"],["colorconfig"]],["bool"]]],[11,"eq","","",2,[[["self"],["outputformat"]],["bool"]]],[11,"eq","","",6,[[["exitstatus"],["self"]],["bool"]]],[11,"ne","","",6,[[["exitstatus"],["self"]],["bool"]]],[11,"fmt","","",0,[[["formatter"],["self"]],["result"]]],[11,"fmt","","",1,[[["formatter"],["self"]],["result"]]],[11,"fmt","","",2,[[["formatter"],["self"]],["result"]]],[11,"fmt","","",3,[[["formatter"],["self"]],["result"]]],[11,"fmt","","",5,[[["formatter"],["self"]],["result"]]],[11,"fmt","","",6,[[["formatter"],["self"]],["result"]]],[11,"from_str","","",1,[[["str"]],["result"]]],[11,"from_str","","",2,[[["str"]],["result"]]]],"p":[[3,"Args"],[4,"ColorConfig"],[4,"OutputFormat"],[3,"TestDesc"],[3,"Test"],[3,"Outcome"],[3,"ExitStatus"]]};
addSearchOptions(searchIndex);initSearch(searchIndex);