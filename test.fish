#!/usr/bin/env fish
if test -d ./output
    echo "Do you want to trash the existing './output' directory? (y/N) "
    read -n 1 -l response

    if test "$response" = y -o "$response" = Y
        echo "Trashing './output'..."
        trash ./output
    else
        echo "Keeping existing './output' directory."
    end
end

function run_task
    set category $argv[1]
    set user $argv[2]
    set extra_args $argv[3..]

    set output_dir "./output/$category"
    set output_file "$output_dir/$user"
    set error_file "$output_file.err"

    mkdir -p $output_dir

    begin
        if timeout 30 cargo run -q --features=stateful $extra_args --example $category -- $user >$output_file 2>$error_file
            echo "$user: $category ✅ SUCCESS"
        else
            if test $status -eq 124
                echo "$user: $category ⏳ TIMED OUT"
            else
                if test "$category" = wrapper
                    if string match -q '*Error: "No hoyos found"*' (cat $error_file)
                        echo "$user: $category ✅ SUCCESS (no hoyos found)"
                    end
                else
                    echo "$user: $category ❌ FAILED"
                end
            end
        end
    end &
end

set users TheSast jxtq Mero Algoinde ExLin YOUNGONE aether seria_ati Mimee GordonCheng tankload orion ya rinn Syd Kaihan VxmpxreFxngs Nolo mathsmagician Vaza54 Shima_ZZZ Misaki tabsy Aesty Yoruhime Hedanman deafly Snaccc britt.pknapp theWsauce Botrecian Ize_Twilight broseph Utsutsun Ricecakes swlf Achira ICE1ICE Liggly
set user_count 0

if [ -z $argv[1] ]
    set argv[1] 5
end

set output_dir "./output/$category"
set tmp_output_file "$output_dir/latest.log"
mkdir -p $output_dir
touch $tmp_output_file

for user in $users
    if [ $user_count -ge $argv[1] ]
        break
    end
    set user_count (math $user_count + 1)
    run_task profile $user | tee -a $tmp_output_file
    run_task hoyos $user | tee -a $tmp_output_file
    run_task wrapper $user --features=stateful | tee -a $tmp_output_file
end

printf "\e["(math $user_count '*' 3)"A"
cat $tmp_output_file | column -t
rm $tmp_output_file

wait
echo "All tasks completed."
