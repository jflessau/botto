define table reminder schemafull;
define field room_id on table reminder type string assert string::len($value) >= 6;
define field title on table reminder type string assert string::len($value) >= 1;
define field interval_unit on table reminder type string assert $value in ['minute', 'hour', 'day'];
define field min_interval on table reminder type int assert $value > 0;
define field max_interval on table reminder type option<int> assert $value > 0 and $value >= $this.min_interval;
define field recurring on table reminder type bool;
define field last_sent_at on table reminder type option<datetime>; 
define field next_send_at on table reminder type option<datetime>;
define field created_at on table reminder type datetime default time::now();

define event set_next_send_at on table reminder
  when $event = "CREATE"
  then (
    update reminder set next_send_at = fn::next_send_at($after.interval_unit, $after.min_interval, $after.max_interval) where id = $after.id
  );

define function fn::max_interval($min: int, $max: option<int>) {
  return if ($max is not none) {
    return $max;
  } else {
    return $min;
  };
};

define function fn::next_send_at($interval_unit: string, $min_interval: int, $max_interval: option<int>) {
  let $next_send_at = time::now();
  let $minimum_interval = $min_interval;
  let $maximum_interval = fn::max_interval($min_interval, $max_interval);

  let $range_start = if $interval_unit == 'day' {
    return duration::from::days($minimum_interval);
  } else if $interval_unit == 'hour' {
    return duration::from::hours($minimum_interval);
  } else if $interval_unit == 'minute' {
    return duration::from::mins($minimum_interval);
  } else {
    throw "interval_unit is not valid: " + $interval_unit;
  };

  let $range_end = if $interval_unit == 'day' {
    return duration::from::days($maximum_interval);
  } else if $interval_unit == 'hour' {
    return duration::from::hours($maximum_interval);
  } else if $interval_unit == 'minute' {
    return duration::from::mins($maximum_interval);
  } else {
    throw "interval_unit is not valid: " + $interval_unit;
  };

  $range_start_datetime = time::now() + $range_start;
  $range_end_datetime = time::now() + $range_end;

  return rand::time(time::unix($range_start_datetime), time::unix($range_end_datetime));
};

define function fn::send_reminder($reminder: record<reminder>) {
  if $reminder.next_send_at is none {
    throw "reminder has no next_send_at value";
  } else if $reminder.next_send_at > time::now() {
    throw "reminder next_send_at is in the future";
  } else if !($reminder.recurring || $reminder.last_sent_at is none) {
    throw "reminder is either not recurring or last_sent_at is not none, last_sent_at: " + <string>$reminder.last_sent_at;
  } else {
    let $next_send_at = if $reminder.recurring {
      return fn::next_send_at($reminder.interval_unit, $reminder.min_interval, $reminder.max_interval);
    } else {
      return none;
    };

    if $next_send_at is none {
      delete $reminder;
    } else {
      update $reminder set
        last_sent_at = time::now(),
        next_send_at = $next_send_at;
      }
    }
};
